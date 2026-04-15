---
name: rust-backend
description: Patterns for the Moestuin Axum + Toasty backend — auth, SSE, Toasty models, insta snapshots, compression. Use whenever editing files under server/.
---

# Rust backend (Moestuin)

Invoke this skill when working in `server/`.

## Stack reminders

- **Axum 0.7+** on **Tokio**. Handlers are `async fn`, extract via typed extractors.
- **Toasty** is the ORM over **SQLite**. Define models in `server/src/models/`; migrations live in `server/migrations/` and run on startup.
- **axum-security** supplies session middleware; cookies are `HttpOnly`, `Secure`, `SameSite=Lax`.
- **tower-http** `CompressionLayer::new()` (gzip + br) wraps the router.
- **tracing** + `tracing-subscriber` with JSON output in release, pretty in debug.

## Router shape

```rust
Router::new()
    .nest("/auth", auth::router())
    .nest("/api", api::router().layer(require_auth()))
    .layer(CompressionLayer::new())
    .layer(TraceLayer::new_for_http())
    .with_state(app_state)
```

The `require_auth` layer rejects anonymous requests with 401. Never attach it inside `/auth`.

## OIDC + allowlist

- Google OIDC via `openidconnect` crate.
- On callback: verify ID token, check `email_verified == true`, check email is in `config.allowed_emails`. Reject otherwise.
- Create a session row, set a signed cookie with the session id.

## SSE

```rust
async fn readings_stream(State(s): State<AppState>)
    -> Sse<impl Stream<Item = Result<Event, Infallible>>>
{
    let rx = s.reading_bus.subscribe();
    let stream = BroadcastStream::new(rx)
        .filter_map(|r| async move { r.ok() })
        .map(|r| Ok(Event::default().event("reading").json_data(r).unwrap()));
    Sse::new(stream).keep_alive(KeepAlive::new().interval(Duration::from_secs(15)))
}
```

Sensor polling task publishes to a `tokio::sync::broadcast` channel; handlers subscribe.

## Hardware fallback

All hardware access goes through a trait (`SensorDriver`, `PumpDriver`, `Webcam`). Select real vs. mock impl at startup based on `MOESTUIN_MOCK_HW` env var so dev + CI work off-Pi.

## Testing

- **insta** snapshots for every JSON endpoint. Redact timestamps/ids:
  ```rust
  insta::assert_json_snapshot!(body, { ".createdAt" => "[ts]", ".id" => "[id]" });
  ```
- Integration tests spin up the router with an in-memory SQLite and mock drivers.
- Run: `cargo test`, review snapshots with `cargo insta review`.

## Errors

Single `AppError` enum with `IntoResponse`. Shape:

```json
{ "error": { "code": "UNAUTHORIZED", "message": "..." } }
```

Never leak internal error messages to clients — log details via `tracing::error!`, return a generic message.

## Don't

- Don't write raw SQL when Toasty can express it.
- Don't block the runtime — wrap sync hardware calls in `tokio::task::spawn_blocking`.
- Don't disable compression for JSON endpoints.
- Don't accept unauthenticated writes, ever.
