//! Test harness: a real `eplyx-cloud` server on loopback over a fresh Postgres
//! database per test. Requires `EPLYX_CLOUD_TEST_DATABASE_URL` pointing at a
//! server where the user may create databases; `make test-cloud` runs these.
#![allow(dead_code)]
use eplyx_lifecycle_impact::{
    cloud::contract::{self, CounterexampleDocument, ReproductionDocument, RunDocument},
    dashboard::store::Store,
    repo_root,
};
use reqwest::blocking::{Client, Response};
use serde_json::{json, Value};
use std::{
    fs,
    path::{Path, PathBuf},
    sync::atomic::{AtomicU64, Ordering},
    time::Duration,
};

pub const HEALTHY: &str = "run_20260924122823725_ce7b4d55310c";
pub const UNDERFUNDED: &str = "run_20260924123736483_ce7b4d55310c";
pub const RESERVE_BOUNDARY: &str = "cx_2ab4735232a3f228da1a29d1";
pub const LOCAL_ID: &str = "project_bccbf1104fb8e3094e85";

static NEXT: AtomicU64 = AtomicU64::new(0);

fn admin_url() -> String {
    std::env::var("EPLYX_CLOUD_TEST_DATABASE_URL").expect(
        "set EPLYX_CLOUD_TEST_DATABASE_URL to a Postgres URL (see `make test-cloud`); cloud tests never skip silently",
    )
}

fn with_database(url: &str, name: &str) -> String {
    match url.rsplit_once('/') {
        Some((server, _)) => format!("{server}/{name}"),
        None => panic!("database URL must end with /<database>"),
    }
}

async fn admin(sql: &str) {
    let (client, connection) = tokio_postgres::connect(&admin_url(), tokio_postgres::NoTls)
        .await
        .expect("connect to the test Postgres server");
    tokio::spawn(connection);
    client.batch_execute(sql).await.expect(sql);
}

pub struct Server {
    pub base: String,
    database: String,
    runtime: Option<tokio::runtime::Runtime>,
}

impl Server {
    pub fn start() -> Self {
        Self::start_with(|_| {})
    }

    pub fn start_with(configure: impl FnOnce(&mut eplyx_cloud::Config)) -> Self {
        let database = format!(
            "eplyx_test_{}_{}_{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed),
            chrono::Utc::now().timestamp_micros()
        );
        let runtime = tokio::runtime::Builder::new_multi_thread()
            .worker_threads(4)
            .enable_all()
            .build()
            .unwrap();
        runtime.block_on(admin(&format!("CREATE DATABASE {database}")));
        let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        let port = listener.local_addr().unwrap().port();
        let base = format!("http://127.0.0.1:{port}");
        let mut config = eplyx_cloud::Config {
            database_url: with_database(&admin_url(), &database),
            public_url: base.clone(),
            bind: listener.local_addr().unwrap(),
            signup_code: None,
            demo_project: None,
        };
        configure(&mut config);
        listener.set_nonblocking(true).unwrap();
        runtime.spawn(async move {
            let app = eplyx_cloud::app(config).await.expect("start eplyx-cloud");
            let listener = tokio::net::TcpListener::from_std(listener).unwrap();
            axum::serve(listener, app).await.unwrap();
        });
        let server = Self {
            base,
            database,
            runtime: Some(runtime),
        };
        for _ in 0..100 {
            if reqwest::blocking::get(format!("{}/healthz", server.base))
                .is_ok_and(|r| r.status().is_success())
            {
                return server;
            }
            std::thread::sleep(Duration::from_millis(50));
        }
        panic!("eplyx-cloud did not become healthy");
    }

    pub fn url(&self, path: &str) -> String {
        format!("{}{path}", self.base)
    }

    /// Run SQL against this test's database (to prove DB-level guarantees).
    pub fn sql(&self, sql: &str) -> Result<(), String> {
        let url = with_database(&admin_url(), &self.database);
        self.runtime.as_ref().unwrap().block_on(async move {
            let (client, connection) = tokio_postgres::connect(&url, tokio_postgres::NoTls)
                .await
                .map_err(|e| e.to_string())?;
            tokio::spawn(connection);
            client
                .batch_execute(sql)
                .await
                .map_err(|e| format!("{e:?}"))
        })
    }

    /// A browser: cookie jar and this site's Origin on every request.
    pub fn browser(&self) -> Client {
        let mut headers = reqwest::header::HeaderMap::new();
        headers.insert("origin", self.base.parse().unwrap());
        Client::builder()
            .cookie_store(true)
            .default_headers(headers)
            .redirect(reqwest::redirect::Policy::none())
            .build()
            .unwrap()
    }

    pub fn signup(&self, email: &str) -> (Client, Value) {
        let browser = self.browser();
        let response = browser
            .post(self.url("/api/v1/auth/signup"))
            .json(&json!({"email": email, "password": "correct horse battery", "name": email.split('@').next().unwrap()}))
            .send()
            .unwrap();
        assert_eq!(response.status(), 201, "signup {email}");
        let body: Value = response.json().unwrap();
        (browser, body)
    }

    /// CLI device flow approved by `browser`; returns a user bearer token.
    pub fn device_token(&self, browser: &Client) -> String {
        let anonymous = Client::new();
        let start: Value = anonymous
            .post(self.url("/api/v1/auth/device"))
            .json(&json!({"client": "test CLI"}))
            .send()
            .unwrap()
            .json()
            .unwrap();
        let approve = browser
            .post(self.url("/api/v1/auth/device/approve"))
            .json(&json!({"user_code": start["user_code"], "approve": true}))
            .send()
            .unwrap();
        assert_eq!(approve.status(), 200);
        let token: Value = anonymous
            .post(self.url("/api/v1/auth/device/token"))
            .json(&json!({"device_code": start["device_code"]}))
            .send()
            .unwrap()
            .json()
            .unwrap();
        token["access_token"].as_str().unwrap().to_owned()
    }

    pub fn create_project(&self, browser: &Client, workspace: &str, name: &str) -> String {
        let response: Value = browser
            .post(self.url(&format!("/api/v1/workspaces/{workspace}/projects")))
            .json(&json!({"name": name}))
            .send()
            .unwrap()
            .json()
            .unwrap();
        response["project"]["id"].as_str().unwrap().to_owned()
    }
}

impl Drop for Server {
    fn drop(&mut self) {
        if let Some(runtime) = self.runtime.take() {
            runtime.shutdown_timeout(Duration::from_secs(2));
        }
        let database = self.database.clone();
        let _ = std::thread::spawn(move || {
            tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .unwrap()
                .block_on(admin(&format!(
                    "DROP DATABASE IF EXISTS {database} WITH (FORCE)"
                )));
        })
        .join();
    }
}

pub fn bearer(token: &str) -> Client {
    let mut headers = reqwest::header::HeaderMap::new();
    headers.insert("authorization", format!("Bearer {token}").parse().unwrap());
    Client::builder().default_headers(headers).build().unwrap()
}

pub fn status(response: Response) -> (u16, Value) {
    let code = response.status().as_u16();
    (code, response.json().unwrap_or(Value::Null))
}

fn copy_tree(from: &Path, to: &Path) {
    fs::create_dir_all(to).unwrap();
    for entry in fs::read_dir(from).unwrap() {
        let entry = entry.unwrap();
        let target = to.join(entry.file_name());
        if entry.file_type().unwrap().is_dir() {
            copy_tree(&entry.path(), &target);
        } else {
            fs::copy(entry.path(), target).unwrap();
        }
    }
}

/// A private copy of the Milestone 16 dashboard fixture project.
pub fn fixture_project() -> PathBuf {
    let root = std::env::temp_dir().join(format!(
        "eplyx-cloud-test-{}-{}",
        std::process::id(),
        NEXT.fetch_add(1, Ordering::Relaxed)
    ));
    let _ = fs::remove_dir_all(&root);
    copy_tree(
        &repo_root().join("fixtures/dashboard/transition-acceptance"),
        &root,
    );
    for child in ["runs", "counterexamples", "reproductions", "cache"] {
        fs::create_dir_all(root.join(".eplyx").join(child)).unwrap();
    }
    fs::write(root.join("eplyx.toml"), "").unwrap();
    root.canonicalize().unwrap()
}

pub struct Documents {
    pub runs: Vec<RunDocument>,
    pub counterexamples: Vec<CounterexampleDocument>,
    pub reproductions: Vec<ReproductionDocument>,
}

pub fn documents(root: &Path, local_id: &str) -> Documents {
    let store = Store::open(root).unwrap();
    let runs = store
        .run_ids()
        .unwrap()
        .0
        .iter()
        .map(|id| contract::run_document(&store, local_id, id).unwrap())
        .collect();
    let counterexamples = store
        .counterexample_ids()
        .unwrap()
        .0
        .iter()
        .map(|id| contract::counterexample_document(&store, local_id, id).unwrap())
        .collect();
    let reproductions = store
        .reproduction_ids()
        .unwrap()
        .0
        .iter()
        .map(|id| contract::reproduction_document(&store, local_id, id, &[]).unwrap())
        .collect();
    Documents {
        runs,
        counterexamples,
        reproductions,
    }
}

/// Sync every fixture document with `client`; panics on any non-2xx answer.
pub fn sync_all(server: &Server, client: &Client, project: &str, docs: &Documents) {
    for run in &docs.runs {
        let (code, body) = status(
            client
                .post(server.url(&format!("/api/v1/projects/{project}/runs")))
                .json(run)
                .send()
                .unwrap(),
        );
        assert!(code == 200 || code == 201, "{code} {body}");
    }
    for cx in &docs.counterexamples {
        let (code, body) = status(
            client
                .post(server.url(&format!("/api/v1/projects/{project}/counterexamples")))
                .json(cx)
                .send()
                .unwrap(),
        );
        assert!(code == 200 || code == 201, "{code} {body}");
    }
    for repro in &docs.reproductions {
        let (code, body) = status(
            client
                .post(server.url(&format!("/api/v1/projects/{project}/reproductions")))
                .json(repro)
                .send()
                .unwrap(),
        );
        assert!(code == 200 || code == 201, "{code} {body}");
    }
}
