use anyhow::{bail, ensure, Context, Result};
use reqwest::blocking::Client;
use serde_json::{json, Value};
use std::io::Read;
use std::time::Duration;

/// Returns full JSON-RPC result values; context and raw bytes are retained by the source.
pub trait SolanaRpc {
    fn call(&self, method: &str, params: Value) -> Result<Value>;
    /// Provider origin only; credentials, paths and query strings never enter snapshots.
    fn origin(&self) -> String;
}

pub struct HttpSolanaRpc {
    client: Client,
    url: reqwest::Url,
    bounded: bool,
    response_limit: u64,
    /// Attempts per request. A long sequential scan needs more patience with a
    /// rate-limited provider than a single small observation does.
    attempts: u32,
}

impl HttpSolanaRpc {
    pub fn new(url: &str) -> Result<Self> {
        let url = reqwest::Url::parse(url).context("invalid RPC URL")?;
        ensure!(
            matches!(url.scheme(), "https" | "http"),
            "RPC must use HTTP(S)"
        );
        let client = Client::builder().timeout(Duration::from_secs(60)).build()?;
        Ok(Self {
            client,
            url,
            bounded: false,
            response_limit: 2 * 1024 * 1024,
            attempts: 6,
        })
    }

    /// Captured deployed code needs a larger but still bounded response budget.
    pub fn bounded_execution(url: &str) -> Result<Self> {
        let mut rpc = Self::bounded(url)?;
        rpc.response_limit = 16 * 1024 * 1024;
        Ok(rpc)
    }

    /// One bounded current-population enumeration. The response ceiling and the
    /// request timeout are server-supplied budget values; nothing from a browser
    /// reaches this constructor.
    pub fn bounded_population(
        url: &str,
        response_limit: u64,
        timeout_seconds: u64,
    ) -> Result<Self> {
        let mut rpc = Self::new(url)?;
        rpc.client = Client::builder()
            .timeout(Duration::from_secs(timeout_seconds))
            .redirect(reqwest::redirect::Policy::none())
            .build()?;
        rpc.bounded = true;
        rpc.response_limit = response_limit;
        Ok(rpc)
    }

    /// Small on-demand observations: no redirects, two attempts, bounded response bytes.
    pub fn bounded(url: &str) -> Result<Self> {
        let mut rpc = Self::new(url)?;
        rpc.client = Client::builder()
            .timeout(Duration::from_secs(12))
            .redirect(reqwest::redirect::Policy::none())
            .build()?;
        rpc.bounded = true;
        rpc.attempts = 2;
        Ok(rpc)
    }
}

impl SolanaRpc for HttpSolanaRpc {
    fn origin(&self) -> String {
        self.url.origin().ascii_serialization()
    }

    fn call(&self, method: &str, params: Value) -> Result<Value> {
        let attempts = self.attempts;
        for attempt in 0..attempts {
            // Pacing keeps a full authority scan within public RPC request limits.
            std::thread::sleep(Duration::from_millis(250));
            let response = self
                .client
                .post(self.url.clone())
                .json(&json!({"jsonrpc":"2.0", "id":1, "method":method, "params":params}))
                .send()
                .map_err(|e| e.without_url())
                .context(format!("RPC {method} transport failed"))?;
            let status = response.status();
            if (status.as_u16() == 429 || status.is_server_error()) && attempt + 1 < attempts {
                std::thread::sleep(Duration::from_secs(1 << attempt));
                continue;
            }
            ensure!(status.is_success(), "RPC {method} HTTP status {status}");
            let body: Value = if self.bounded {
                let mut bytes = Vec::new();
                response
                    .take(self.response_limit + 1)
                    .read_to_end(&mut bytes)
                    .context("RPC response read failed")?;
                ensure!(
                    bytes.len() as u64 <= self.response_limit,
                    "RPC response exceeds observation budget"
                );
                serde_json::from_slice(&bytes).context("invalid RPC JSON")?
            } else {
                response
                    .json()
                    .map_err(|e| e.without_url())
                    .context("invalid RPC JSON")?
            };
            ensure!(
                body["jsonrpc"] == "2.0" && body["id"] == 1,
                "invalid JSON-RPC envelope"
            );
            if let Some(error) = body.get("error") {
                let code = error["code"].as_i64();
                if matches!(code, Some(429 | -32005 | -32016)) && attempt + 1 < attempts {
                    std::thread::sleep(Duration::from_secs(1 << attempt));
                    continue;
                }
                // Provider error text can contain URLs/credentials; report only the code.
                bail!("RPC {method} failed with code {code:?}");
            }
            return body.get("result").cloned().context("RPC result missing");
        }
        bail!("RPC {method} exhausted retries")
    }
}
