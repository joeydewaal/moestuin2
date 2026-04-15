//! Auth integration tests. Use the mock-auth dev path to avoid talking to Google.
//!
//! These spin up the real binary with a random port so we can exercise the full
//! cookie round-trip.

use std::{
    net::{SocketAddr, TcpListener},
    process::{Child, Command, Stdio},
    time::{Duration, Instant},
};

fn pick_port() -> u16 {
    let l = TcpListener::bind("127.0.0.1:0").unwrap();
    l.local_addr().unwrap().port()
}

struct Server {
    child: Child,
    addr: SocketAddr,
}

impl Server {
    fn spawn() -> Self {
        let port = pick_port();
        let addr: SocketAddr = format!("127.0.0.1:{port}").parse().unwrap();

        let exe = env!("CARGO_BIN_EXE_server");
        let child = Command::new(exe)
            .env("MOESTUIN_MOCK_AUTH", "1")
            .env("MOESTUIN_MOCK_HW", "1")
            .env("MOESTUIN_BIND", addr.to_string())
            .env("MOESTUIN_ALLOWED_EMAILS", "family@example.com")
            .env("MOESTUIN_DATABASE_URL", "sqlite::memory:")
            .env("RUST_LOG", "warn")
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .expect("spawn server");

        let deadline = Instant::now() + Duration::from_secs(10);
        let client = reqwest::blocking::Client::new();
        loop {
            if let Ok(r) = client.get(format!("http://{addr}/health")).send()
                && r.status().is_success()
            {
                break;
            }
            if Instant::now() > deadline {
                panic!("server didn't start in time");
            }
            std::thread::sleep(Duration::from_millis(100));
        }

        Self { child, addr }
    }

    fn url(&self, path: &str) -> String {
        format!("http://{}{}", self.addr, path)
    }
}

impl Drop for Server {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

#[test]
fn me_requires_auth_then_returns_user_after_dev_login() {
    let server = Server::spawn();
    let client = reqwest::blocking::Client::builder()
        .cookie_store(true)
        .redirect(reqwest::redirect::Policy::none())
        .build()
        .unwrap();

    // unauthenticated → 401
    let resp = client.get(server.url("/auth/me")).send().unwrap();
    assert_eq!(resp.status(), 401);

    // unknown email → 403
    let resp = client
        .get(server.url("/auth/dev-login?email=stranger@example.com"))
        .send()
        .unwrap();
    assert_eq!(resp.status(), 403);

    // allow-listed email → redirect + cookie
    let resp = client
        .get(server.url("/auth/dev-login?email=family@example.com&name=Family"))
        .send()
        .unwrap();
    assert!(resp.status().is_redirection() || resp.status().is_success());

    // /api/ping should also work
    let resp = client.get(server.url("/api/ping")).send().unwrap();
    assert_eq!(resp.status(), 200);

    // now /auth/me should work
    let resp = client.get(server.url("/auth/me")).send().unwrap();
    assert_eq!(resp.status(), 200);
    let mut body: serde_json::Value = resp.json().unwrap();
    // redact the uuid so the snapshot is stable
    if let Some(id) = body.get_mut("id") {
        *id = serde_json::Value::String("[uuid]".into());
    }
    insta::assert_json_snapshot!(body);
}
