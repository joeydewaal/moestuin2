# Moestuin

A garden/allotment tracker self-hosted on a Raspberry Pi. Family-only website showing live sensor data, webcam feed, crop records, and water-pump controls.

## Purpose

- Monitor allotment conditions (temperature + soil moisture) in real time.
- Provide a 24/7 webcam feed plus a rolling archive of past days.
- Log what crops are planted, when, and what was harvested, with photos.
- Control and schedule water pumps remotely.

## Architecture

```
┌─────────────────┐    HTTPS / SSE      ┌──────────────────────┐
│  Svelte 5 PWA   │ ◄──────────────────►│  Rust / Axum server  │
│  (Skeleton UI)  │       cookies       │  (Tokio, Toasty)     │
└─────────────────┘                     └─────────┬────────────┘
                                                  │
                                        ┌─────────┼─────────────┐
                                        ▼         ▼             ▼
                                    SQLite     GPIO          Webcam
                                              (sensors,      (ffmpeg
                                              pumps)         segments)
```

Everything runs on one Raspberry Pi. Reverse-proxy (Caddy or nginx) terminates TLS and serves the built frontend + proxies `/api` to Axum.

## Tech stack

### Frontend (`/web`)
- **Svelte 5** (runes API — `$state`, `$derived`, `$effect`).
- **TanStack Query** (`@tanstack/svelte-query`) for all data fetching; SSE feeds a query cache.
- **Skeleton UI** for components + themes (light/dark toggle).
- **PWA** — installable, offline shell, service worker.
- **Vitest + Playwright** for unit and UI tests. Every new feature ships a Playwright test.
- **Spinners** on every loading state (use Skeleton's `ProgressRing`/`Spinner`).
- Google sign-in button is the only auth entry point.

### Backend (`/server`)
- **Rust** edition 2024, **Tokio**, **Axum**.
- **Toasty** as the ORM over **SQLite**.
- **axum-security** for middleware, cookie-based sessions (HttpOnly, Secure, SameSite=Lax).
- **OIDC** (Google) with a hard-coded allowlist of family emails in config.
- **insta** for snapshot tests of API responses.
- Response **compression** via `tower-http` (`CompressionLayer`: gzip + br).
- Data endpoints are gated behind auth middleware; only `/auth/*` and health are public.

### Hardware
- DHT22 or similar for temp + humidity.
- Capacitive soil-moisture sensors (analog → ADS1115 → I²C).
- Relay-controlled 12V water pumps on GPIO.
- USB webcam captured with `ffmpeg` into 5-minute segments; daily rollup.

### CI
- GitHub Actions (or Forgejo if self-hosted).
- Jobs: `fmt` (cargo fmt + prettier), `lint` (clippy + eslint), `test` (cargo test + vitest + playwright headless).
- Must pass before merge.

## Repo layout

```
moestuin/
├── CLAUDE.md
├── ROADMAP.md
├── .claude/skills/        # project-specific skills
├── web/                   # Svelte 5 PWA
├── server/                # Axum backend
│   ├── src/
│   ├── migrations/
│   └── tests/             # insta snapshots live here
├── hardware/              # sensor/pump/webcam daemons (may live inside server/)
├── deploy/                # systemd units, Caddyfile, install scripts
└── .github/workflows/
```

## Conventions

- **Secrets** live in `.env` (gitignored) and are loaded via `dotenvy`. The email allowlist is in `server/config.toml`.
- **Database migrations** are checked-in SQL under `server/migrations/`, applied on startup.
- **API shape**: JSON, `camelCase` on the wire (serde rename_all).
- **Errors**: backend returns `{ "error": { "code": "...", "message": "..." } }`; frontend surfaces via a Skeleton toast.
- **Time** is stored UTC (`chrono::DateTime<Utc>`), rendered in Europe/Amsterdam on the client.
- **Logging**: `tracing` on the backend, structured JSON in production.

## Running locally

```bash
# backend
cd server && cargo run

# frontend
cd web && pnpm dev
```

The backend mocks GPIO/sensors/webcam on non-Linux or when `MOESTUIN_MOCK_HW=1`.

## Deployment

- `deploy/install.sh` builds release binary, copies systemd units, enables `moestuin.service` and `moestuin-webcam.service`.
- Webcam archive lives in `/var/lib/moestuin/webcam/YYYY-MM-DD/`.
- SQLite DB at `/var/lib/moestuin/moestuin.db`; nightly `sqlite3 .backup` to `/var/lib/moestuin/backups/`.

## Skills

Project-specific skills in `.claude/skills/`:
- `svelte-frontend` — Svelte 5 runes, TanStack Query, Skeleton UI patterns used here.
- `rust-backend` — Axum + Toasty + auth patterns used here.
- `pi-hardware` — sensors, pumps, webcam on the Pi with mock fallbacks.
- `ci-checks` — formatting, linting, and test commands.
