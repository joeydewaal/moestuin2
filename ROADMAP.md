# Roadmap

Milestones are ordered so the site is usable end-to-end as early as possible, then hardened.

## M0 — Scaffolding
- [ ] Init `server/` (`cargo new`), add Tokio, Axum, Toasty, tower-http, axum-security, tracing, serde, chrono, insta.
- [ ] Init `web/` (`pnpm create svelte`), add TanStack Query, Skeleton UI, Playwright, Vitest, PWA plugin.
- [ ] GitHub Actions: `fmt`, `lint`, `test` workflows for both projects.
- [ ] `deploy/` skeleton: Caddyfile, systemd unit templates.
- [ ] CLAUDE.md, README, pre-commit hooks (`cargo fmt`, `prettier`).

## M1 — Auth
- [ ] Google OIDC on the backend, with email allowlist from `config.toml`.
- [ ] Session cookies (HttpOnly, Secure, SameSite=Lax), CSRF token on POST routes.
- [ ] `/auth/login`, `/auth/callback`, `/auth/logout`, `/auth/me`.
- [ ] Auth middleware gates every `/api/*` endpoint.
- [ ] Frontend: Google sign-in screen, `useSession` query, redirect logic.
- [ ] Playwright test: unauthenticated user is bounced to login.
- [ ] insta snapshot of `/auth/me` response.

## M2 — Sensors + live graph
- [ ] Sensor daemon task (`tokio::spawn`) polls DHT22 + moisture every 30s, writes to `readings` table.
- [ ] Mock sensor driver behind `MOESTUIN_MOCK_HW`.
- [ ] `GET /api/readings?from=..&to=..` (paginated).
- [ ] `GET /api/readings/stream` (SSE) — pushes new readings as they land.
- [ ] Frontend: dashboard page with line graph (e.g. Chart.js or LayerCake) fed by TanStack Query, kept live via SSE. Spinner on first load.
- [ ] Light/dark theme toggle wired to Skeleton themes, persisted in `localStorage`.
- [ ] Playwright: mocked SSE updates the graph.

## M3 — Webcam
- [ ] `moestuin-webcam` service: ffmpeg captures HLS (`.m3u8` + segments), rotates into `webcam/YYYY-MM-DD/`.
- [ ] Nightly job compresses yesterday's segments to a single daily MP4, keeps last 30 days configurable.
- [ ] `GET /api/webcam/live` returns the HLS playlist URL.
- [ ] `GET /api/webcam/archive` lists dates; `GET /api/webcam/archive/:date` returns playback URL.
- [ ] Frontend: live view (hls.js), date picker for archive, spinners while loading.
- [ ] Playwright: live tile renders, archive date selection swaps player source.

## M4 — Crops
- [ ] Tables: `crops`, `crop_events` (planted/harvested/note), `crop_photos`.
- [ ] CRUD endpoints + photo upload (multipart → disk under `/var/lib/moestuin/photos/`).
- [ ] Frontend: crop list, crop detail page with timeline + photo gallery, add/edit forms.
- [ ] Harvest record captures weight/quantity + notes.
- [ ] insta snapshots for each endpoint; Playwright flow for "plant a crop → upload photo → record harvest".

## M5 — Water pumps
- [ ] Tables: `pumps`, `pump_schedules`, `pump_runs`.
- [ ] GPIO pump driver + mock.
- [ ] Scheduler task reads `pump_schedules` (cron-like) and triggers pumps, logging each `pump_run` (duration, volume estimate).
- [ ] `POST /api/pumps/:id/run` (manual trigger, body: duration).
- [ ] `GET /api/pumps`, `PUT /api/pumps/:id/schedule`.
- [ ] Frontend: pump card with manual run button (confirm dialog), schedule editor, run history table.
- [ ] Safety: min interval between runs, max duration cap, emergency stop endpoint.
- [ ] Playwright: manual run button disables while running, schedule update round-trips.

## M6 — PWA polish
- [ ] Service worker with offline shell (cached dashboard skeleton).
- [ ] Web app manifest with icons, theme color, standalone display.
- [ ] Installable on iOS/Android verified manually.
- [ ] Lighthouse PWA score ≥ 90.

## M7 — Hardening & ops
- [ ] Response compression enabled and verified (gzip + br).
- [ ] Rate limiting on auth routes.
- [ ] Nightly SQLite backup + webcam archive pruning.
- [ ] Healthcheck endpoint + systemd watchdog.
- [ ] Structured logs shipped to a local file, rotated via logrotate.
- [ ] Document restore procedure in `deploy/README.md`.

## Nice-to-haves (post-v1)
- Multi-zone moisture (one reading per bed).
- Weather forecast integration to skip pump runs before rain.
- Telegram/Signal notifications on sensor anomalies.
- Season-over-season harvest comparison view.
