//! Integration tests for the /api/webcam/* routes and the /webcam ServeDir.
//!
//! Lays out a fake webcam root in a TempDir, points Config::webcam_root at it,
//! and walks the live + archive flows end-to-end through the live HTTP server.

use std::{net::SocketAddr, path::PathBuf};

use server::{build_app, config::Config, open_db};
use tempfile::TempDir;

const TODAY: &str = "2026-05-01";
const YESTERDAY: &str = "2026-04-30";
const DAY_BEFORE: &str = "2026-04-29";

struct TestServer {
    addr: SocketAddr,
    handle: tokio::task::JoinHandle<()>,
    _webcam_dir: TempDir,
}

impl TestServer {
    /// Spawn the server backed by `webcam_root`. Caller controls the on-disk layout.
    async fn spawn(webcam_root: TempDir) -> Self {
        // toasty's pool can drop the idle in-memory connection between
        // `push_schema` and the first real query, losing the schema (see
        // toasty/src/db/pool.rs). Use a tempfile-backed DB so all pool
        // connections see the same on-disk schema.
        let db_path = webcam_root.path().join("test.db");
        let cfg = Config {
            bind: "127.0.0.1:0".into(),
            database_url: format!("sqlite:{}", db_path.display()),
            mock_hardware: true,
            mock_auth: true,
            allowed_emails: vec!["family@example.com".into()],
            cookie_secret: b"dev-cookie-secret-do-not-use-in-prod-32".to_vec(),
            oidc: server::config::OidcConfig {
                client_id: String::new(),
                client_secret: String::new(),
                redirect_url: "http://localhost/auth/callback".into(),
                post_logout_redirect_url: "/".into(),
            },
            webcam_root: webcam_root.path().to_path_buf(),
        };

        let db = open_db(&cfg).await.expect("open db");
        let app = build_app(&cfg, db).await.expect("build app");
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
            .await
            .expect("bind");
        let addr = listener.local_addr().unwrap();
        let handle = tokio::spawn(async move {
            let _ = axum::serve(listener, app).await;
        });
        Self {
            addr,
            handle,
            _webcam_dir: webcam_root,
        }
    }

    fn url(&self, path: &str) -> String {
        format!("http://{}{}", self.addr, path)
    }
}

impl Drop for TestServer {
    fn drop(&mut self) {
        self.handle.abort();
    }
}

/// `today/` has a live playlist and one segment.
/// `yesterday/` and `day-before/` each have a rolled-up day.mp4.
fn build_fixture() -> TempDir {
    let dir = tempfile::tempdir().expect("tempdir");
    let root: PathBuf = dir.path().to_path_buf();

    for day in [TODAY, YESTERDAY, DAY_BEFORE] {
        std::fs::create_dir_all(root.join(day)).unwrap();
    }
    std::fs::write(root.join(TODAY).join("live.m3u8"), b"#EXTM3U\n").unwrap();
    std::fs::write(root.join(TODAY).join("seg-00.ts"), b"\x00\x00").unwrap();
    std::fs::write(root.join(YESTERDAY).join("day.mp4"), b"yesterday-mp4").unwrap();
    std::fs::write(root.join(DAY_BEFORE).join("day.mp4"), b"day-before-mp4").unwrap();

    dir
}

async fn login(client: &reqwest::Client, server: &TestServer) {
    let resp = client
        .get(server.url("/auth/dev-login?email=family@example.com&name=Family"))
        .send()
        .await
        .unwrap();
    assert!(resp.status().is_redirection() || resp.status().is_success());
}

fn unauth_client() -> reqwest::Client {
    reqwest::Client::builder()
        .cookie_store(true)
        .redirect(reqwest::redirect::Policy::none())
        .build()
        .unwrap()
}

#[tokio::test]
async fn webcam_endpoints_require_auth() {
    let server = TestServer::spawn(build_fixture()).await;
    let client = unauth_client();

    for path in [
        "/api/webcam/live",
        "/api/webcam/archive",
        "/api/webcam/archive/2026-04-30",
    ] {
        let resp = client.get(server.url(path)).send().await.unwrap();
        assert_eq!(resp.status(), 401, "{path} should require auth");
    }
}

#[tokio::test]
async fn live_returns_most_recent_playlist() {
    let server = TestServer::spawn(build_fixture()).await;
    let client = unauth_client();
    login(&client, &server).await;

    let resp = client
        .get(server.url("/api/webcam/live"))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
    let body: serde_json::Value = resp.json().await.unwrap();
    insta::assert_json_snapshot!(body);
}

#[tokio::test]
async fn live_returns_null_when_no_playlist() {
    let dir = tempfile::tempdir().unwrap();
    let server = TestServer::spawn(dir).await;
    let client = unauth_client();
    login(&client, &server).await;

    let resp = client
        .get(server.url("/api/webcam/live"))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
    let body: serde_json::Value = resp.json().await.unwrap();
    insta::assert_json_snapshot!(body);
}

#[tokio::test]
async fn archive_list_excludes_today_and_is_newest_first() {
    let server = TestServer::spawn(build_fixture()).await;
    let client = unauth_client();
    login(&client, &server).await;

    let resp = client
        .get(server.url("/api/webcam/archive"))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
    let body: serde_json::Value = resp.json().await.unwrap();
    insta::assert_json_snapshot!(body);
}

#[tokio::test]
async fn archive_day_returns_url_when_present() {
    let server = TestServer::spawn(build_fixture()).await;
    let client = unauth_client();
    login(&client, &server).await;

    let resp = client
        .get(server.url(&format!("/api/webcam/archive/{YESTERDAY}")))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
    let body: serde_json::Value = resp.json().await.unwrap();
    insta::assert_json_snapshot!(body);
}

#[tokio::test]
async fn archive_day_404s_for_unknown_date() {
    let server = TestServer::spawn(build_fixture()).await;
    let client = unauth_client();
    login(&client, &server).await;

    let resp = client
        .get(server.url("/api/webcam/archive/2020-01-01"))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 404);
}

#[tokio::test]
async fn archive_day_400s_for_bad_date() {
    let server = TestServer::spawn(build_fixture()).await;
    let client = unauth_client();
    login(&client, &server).await;

    for bad in ["2020-13-01", "yesterday", "2020-1-1"] {
        let resp = client
            .get(server.url(&format!("/api/webcam/archive/{bad}")))
            .send()
            .await
            .unwrap();
        assert_eq!(resp.status(), 400, "{bad} should be rejected");
    }
}

#[tokio::test]
async fn serve_dir_serves_archive_files() {
    let server = TestServer::spawn(build_fixture()).await;
    // Static file serving is not auth-gated; Caddy intercepts in prod, but in
    // dev the Rust server exposes the volume directly. Anyone who can reach
    // the backend (which is gated by Vite proxy + the same allow-listed users)
    // can read frames — same as the prod Caddy mount.
    let client = reqwest::Client::new();

    let resp = client
        .get(server.url(&format!("/webcam/{YESTERDAY}/day.mp4")))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
    assert_eq!(resp.bytes().await.unwrap().as_ref(), b"yesterday-mp4");

    let resp = client
        .get(server.url("/webcam/2020-01-01/day.mp4"))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 404);
}
