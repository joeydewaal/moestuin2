# Roadmap

Milestones are ordered so the site is usable end-to-end as early as possible, then hardened.

## M0 — Scaffolding
- [ ] Init `server/` (`cargo new`), add Tokio, Axum, Toasty, tower-http, axum-security, tracing, serde, chrono, insta.
- [ ] Init `web/` (`pnpm create svelte`), add TanStack Query, Skeleton UI, Playwright, Vitest, PWA plugin.
- [ ] GitHub Actions: `fmt`, `lint`, `test` workflows for both projects.
- [ ] `deploy/` skeleton: Caddyfile + per-service Dockerfiles; root `docker-compose.yml` wiring server, web, webcam, caddy, and a one-shot backup container.
- [ ] CLAUDE.md, README, pre-commit hooks (`cargo fmt`, `prettier`).

## M1 — Auth
- [x] Google OIDC on the backend via `axum-security`, with email allowlist from env (`MOESTUIN_ALLOWED_EMAILS`). Unverified / non-allow-listed emails redirect to `/login?error=…`.
- [x] Session cookies (HttpOnly, `SameSite=Lax`, `Secure` in prod, `Path=/`).
- [x] `/auth/login`, `/auth/callback`, `/auth/logout`, `/auth/me`, plus dev-only `/auth/dev-login` behind `MOESTUIN_MOCK_AUTH`.
- [x] Auth-gated routes use the `CookieSession<User>` extractor (stub `/api/ping` demonstrates it).
- [x] Frontend: Google sign-in page, `sessionQuery` via TanStack Query, `<AuthGuard>` redirect-on-401.
- [x] Playwright test: unauthenticated `/` redirects to `/login`; login button links to `/auth/login`.
- [x] insta snapshot of `/auth/me` response (uuid redacted).
- [ ] CSRF token on POST routes (add when the first mutating endpoint lands).
- [ ] Persistent session store — currently `MemStore`; swap for a SQLite-backed store when we add migrations in M2.

## M2 — Sensors + live graph
- [x] Sensor daemon task (`tokio::spawn`) polls DHT22 + moisture every 30s, writes to `readings` table.
- [x] `SensorDriver` enum (`Real` / `Mock`); startup probes real hardware, auto-falls-back to `Mock` on failure or when `MOESTUIN_MOCK_HW=1`. Active variant surfaced on health endpoint.
- [x] `GET /api/readings?from=..&to=..` (paginated).
- [x] `GET /api/readings/stream` (SSE) — pushes new readings as they land.
- [x] Frontend: dashboard page with line graph (e.g. Chart.js or LayerCake) fed by TanStack Query, kept live via SSE. Spinner on first load.
- [x] Light/dark theme toggle wired to Skeleton themes, persisted in `localStorage`.
- [x] Playwright: mocked SSE updates the graph.

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
- [ ] Healthcheck endpoint + Docker `HEALTHCHECK` directives on every service.
- [ ] Structured logs shipped to a local file, rotated via logrotate.
- [ ] Document restore procedure in `deploy/README.md`.

## M8 — Notifications & alerts
- [ ] Alert rules table (type, threshold, cooldown, enabled).
- [ ] Evaluator task subscribes to sensor broadcast + pump events, emits alerts with dedupe.
- [ ] Delivery channels: email (SMTP) + web push (VAPID) to the PWA. Channel config per user.
- [ ] Frontend: alert rule editor, notification history page, per-device push opt-in.
- [ ] Built-in rules: moisture below threshold, pump run failure, disk < 10% free.

## M9 — Weather integration
- [ ] Open-Meteo client (no API key), cached hourly in SQLite.
- [ ] `GET /api/weather/forecast` surfaces next 24h precipitation + temp.
- [ ] Pump scheduler skips a run if forecast rain ≥ configurable mm in the next N hours; logs the skip reason on `pump_runs`.
- [ ] Frontend: forecast strip on dashboard, "skipped due to rain" badge on pump history.

## M10 — Frost & heat warnings
- [ ] Extend alerting with frost (forecast min ≤ 2°C) and heat (forecast max ≥ configurable) warnings, lead time configurable.
- [ ] Dashboard banner when a warning is active with affected date/time.
- [ ] Per-crop frost sensitivity flag so warnings can be filtered to only what matters.

## M11 — Timelapse
- [ ] Nightly job builds a 24h timelapse MP4 from that day's webcam segments.
- [ ] Per-crop timelapse: stitch the daily files between `planted_at` and `harvested_at` into one video on demand (background job, status surfaced via SSE).
- [ ] Frontend: timelapse tab on each crop, download link, share-safe (signed, time-limited) URL.

## M12 — Mobile-first UX pass
- [ ] Audit every page on a 360px viewport; enforce 44px min tap targets.
- [ ] "Big button" mode for the pump page (one-tap run with haptic confirm via `navigator.vibrate`).
- [ ] Offline-first for dashboard + crop list (TanStack Query persistence to IndexedDB).
- [ ] Camera-capture flow for crop photos uses `<input capture="environment">`.
- [ ] Playwright tests run a mobile viewport project alongside desktop.

## M13 — Audit log
- [ ] `audit_log` table: `actor_user_id`, `action`, `target`, `metadata` JSON, `at` (jiff `Timestamp`).
- [ ] Middleware records all mutating requests; pump runs and schedule edits tagged with the triggering user.
- [ ] `GET /api/audit` (paginated, filterable by actor/action/date).
- [ ] Frontend admin page to browse the log; read-only, only visible to users flagged `is_admin` in config.

## M14 — Off-Pi backups
- [ ] Nightly job uploads SQLite backup + photos + latest timelapses to remote storage (restic to S3/Backblaze B2, or rclone to a second machine — config driven).
- [ ] Retention policy: 7 daily, 4 weekly, 12 monthly.
- [ ] Restore runbook in `deploy/README.md` and a `deploy/restore.sh` helper.
- [ ] Healthcheck fails if the last successful remote backup is older than 48h.

## M15 — Health dashboard
- [ ] `GET /api/health` (public, terse) + `GET /api/health/detail` (auth) reporting: Pi CPU temp, load, disk free, DB size, last sensor reading age per sensor, ffmpeg process up, last backup timestamp, pump driver status.
- [ ] Frontend `/health` page with tiles, red/amber/green status, spinner on load.
- [ ] Alert rules can target any health metric going red.

## Nice-to-haves (post-v1)
- Multi-zone moisture (one reading per bed).
- Season-over-season harvest comparison view.
- Telegram/Signal notification channels in addition to email/web-push.
- Sensor calibration curves per device.
- Sowing calendar + companion planting hints.
