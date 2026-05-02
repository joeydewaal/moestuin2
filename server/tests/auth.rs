//! Auth integration tests. Use the mock-auth dev path to avoid talking to Google.

use std::net::SocketAddr;

use server::{build_app, config::Config, open_db};

struct TestServer {
    addr: SocketAddr,
    handle: tokio::task::JoinHandle<()>,
}

impl TestServer {
    async fn spawn() -> Self {
        let cfg = Config {
            bind: "127.0.0.1:0".into(),
            database_url: "sqlite::memory:".into(),
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
            webcam_root: std::env::temp_dir(),
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
        Self { addr, handle }
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

#[tokio::test]
async fn me_requires_auth_then_returns_user_after_dev_login() {
    let server = TestServer::spawn().await;
    let client = reqwest::Client::builder()
        .cookie_store(true)
        .redirect(reqwest::redirect::Policy::none())
        .build()
        .unwrap();

    // unauthenticated → 401
    let resp = client.get(server.url("/auth/me")).send().await.unwrap();
    assert_eq!(resp.status(), 401);

    // unknown email → 403
    let resp = client
        .get(server.url("/auth/dev-login?email=stranger@example.com"))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 403);

    // allow-listed email → redirect + cookie
    let resp = client
        .get(server.url("/auth/dev-login?email=family@example.com&name=Family"))
        .send()
        .await
        .unwrap();
    assert!(resp.status().is_redirection() || resp.status().is_success());

    // /api/ping should also work
    let resp = client.get(server.url("/api/ping")).send().await.unwrap();
    assert_eq!(resp.status(), 200);

    // now /auth/me should work
    let resp = client.get(server.url("/auth/me")).send().await.unwrap();
    assert_eq!(resp.status(), 200);
    let mut body: serde_json::Value = resp.json().await.unwrap();
    if let Some(id) = body.get_mut("id") {
        *id = serde_json::Value::String("[uuid]".into());
    }
    insta::assert_json_snapshot!(body);
}
