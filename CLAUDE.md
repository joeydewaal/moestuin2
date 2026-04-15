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

Everything runs on one Raspberry Pi, orchestrated by **Docker Compose**. Caddy (in a container) terminates TLS, proxies `/api` + `/auth` to the `server` container, and the rest to the `web` container. Sensors/pumps are accessed by mapping `/dev/gpiomem` and `/dev/i2c-1` into the server container; the webcam container mounts `/dev/video0`.

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
- **jiff** for all date/time handling.
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
├── deploy/                # Caddyfile, webcam + backup container contexts, env example
├── docker-compose.yml
└── .github/workflows/
```

## Conventions

- **Secrets** live in `.env` (gitignored) and are loaded via `dotenvy`. The email allowlist is in `server/config.toml`.
- **Database migrations** are checked-in SQL under `server/migrations/`, applied on startup.
- **API shape**: JSON, `camelCase` on the wire (serde rename_all).
- **Errors**: backend returns `{ "error": { "code": "...", "message": "..." } }`; frontend surfaces via a Skeleton toast.
- **Time** is stored UTC using `jiff` (`jiff::Timestamp`), rendered in Europe/Amsterdam on the client. Do not use `chrono` or `time`.
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

- **Docker Compose** is the only supported deploy path (`docker-compose.yml`). Services: `server`, `web`, `webcam`, `caddy`, and a one-shot `backup` container (profile `tools`).
- Persistent state lives in the `moestuin-data` named volume, mounted at `/data` inside containers: SQLite at `/data/moestuin.db`, webcam archive at `/data/webcam/YYYY-MM-DD/`, photos at `/data/photos/`, backups at `/data/backups/`.
- Hardware access: `server` maps `/dev/gpiomem` + `/dev/i2c-1`; `webcam` maps `/dev/video0`. Host's `gpio` and `i2c` GIDs go into `group_add` — verify with `getent group gpio i2c`.
- Backups are triggered by host cron: `docker compose run --rm backup`.
- Secrets in `deploy/moestuin.env` (gitignored), loaded via compose `env_file`.

## Skills

Project-specific skills in `.claude/skills/`:
- `svelte-frontend` — Svelte 5 runes, TanStack Query, Skeleton UI patterns used here.
- `rust-backend` — Axum + Toasty + auth patterns used here.
- `pi-hardware` — sensors, pumps, webcam on the Pi with mock fallbacks.
- `ci-checks` — formatting, linting, and test commands.
