//! Minimal HTTP client for the Eplyx cloud API. Used only by the cloud CLI
//! commands, never by preflight, search, reproduce, replay or the dashboard.
//! Redirects are refused so the bearer token can never follow to another host.
use anyhow::{anyhow, Result};
use serde::Serialize;
use serde_json::Value;
use std::time::Duration;

pub struct Client {
    server: String,
    token: Option<String>,
    http: reqwest::blocking::Client,
}

/// A cloud call that did not succeed. `Unreachable` covers DNS, connection and
/// timeout failures; nothing local changes and the call can be retried.
#[derive(Debug)]
pub enum CloudError {
    Unreachable(String),
    Status { status: u16, message: String },
}

impl std::fmt::Display for CloudError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Unreachable(detail) => write!(f, "{detail}"),
            Self::Status { status, message } => write!(f, "HTTP {status}: {message}"),
        }
    }
}

impl std::error::Error for CloudError {}

impl Client {
    pub fn new(server: &str, token: Option<String>) -> Result<Self> {
        let http = reqwest::blocking::Client::builder()
            .connect_timeout(Duration::from_secs(10))
            .timeout(Duration::from_secs(90))
            .redirect(reqwest::redirect::Policy::none())
            .user_agent(format!("eplyx/{}", crate::build_info::VERSION))
            .build()?;
        Ok(Self {
            server: server.to_owned(),
            token,
            http,
        })
    }

    pub fn server(&self) -> &str {
        &self.server
    }

    fn send(&self, request: reqwest::blocking::RequestBuilder) -> Result<(u16, Value), CloudError> {
        let request = match &self.token {
            Some(token) => request.bearer_auth(token),
            None => request,
        };
        let response = request.send().map_err(|error| {
            // reqwest errors name the URL (the public server origin), never the token.
            CloudError::Unreachable(format!(
                "could not reach Eplyx cloud at {}: {}",
                self.server,
                error.without_url()
            ))
        })?;
        let status = response.status().as_u16();
        let body = response.text().unwrap_or_default();
        let value: Value = serde_json::from_str(&body).unwrap_or(Value::Null);
        if (200..300).contains(&status) {
            Ok((status, value))
        } else {
            let message = value["error"]
                .as_str()
                .map(str::to_owned)
                .unwrap_or_else(|| format!("unexpected response from {}", self.server));
            Err(CloudError::Status { status, message })
        }
    }

    pub fn get(&self, path: &str) -> Result<Value, CloudError> {
        self.send(self.http.get(format!("{}{path}", self.server)))
            .map(|(_, v)| v)
    }

    pub fn post<T: Serialize + ?Sized>(
        &self,
        path: &str,
        body: &T,
    ) -> Result<(u16, Value), CloudError> {
        self.send(self.http.post(format!("{}{path}", self.server)).json(body))
    }

    pub fn delete(&self, path: &str) -> Result<Value, CloudError> {
        self.send(self.http.delete(format!("{}{path}", self.server)))
            .map(|(_, v)| v)
    }
}

/// Human wording for a failed cloud call, with the retry guidance the CLI prints.
pub fn explain(error: &CloudError) -> anyhow::Error {
    match error {
        CloudError::Unreachable(detail) => anyhow!(
            "{detail}\nNothing local changed. Local preflight, search, reproduce and the dashboard keep working; retry when the cloud is reachable."
        ),
        CloudError::Status { status: 401, .. } => anyhow!(
            "Eplyx cloud rejected the access token (HTTP 401). Run `eplyx login` again, or check EPLYX_TOKEN."
        ),
        CloudError::Status { status, message } => anyhow!("Eplyx cloud: {message} (HTTP {status})"),
    }
}
