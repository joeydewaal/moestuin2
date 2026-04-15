---
name: pi-hardware
description: Raspberry Pi hardware integration for Moestuin — DHT22, soil moisture via ADS1115, GPIO relays for pumps, ffmpeg webcam capture, and mock drivers. Use when touching hardware code or deploy scripts.
---

# Pi hardware (Moestuin)

Invoke when working on sensor daemons, pump drivers, webcam capture, or deploy scripts.

## Target hardware

- Raspberry Pi 4 (64-bit Raspberry Pi OS).
- **DHT22** on a GPIO pin → temperature + humidity.
- **Capacitive soil moisture** sensors (analog) → **ADS1115** ADC → I²C.
- **12V water pumps** driven through a relay board on GPIO (active-low, check your board).
- **USB webcam** at `/dev/video0`.

## Driver enum pattern

Each hardware class is a single enum with `Real` and `Mock` variants — no trait objects, no `Box<dyn>`. Dispatch is a plain `match`:

```rust
pub enum SensorDriver {
    Real(RealSensor),
    Mock(MockSensor),
}

impl SensorDriver {
    pub async fn read(&self) -> Result<Reading> {
        match self {
            Self::Real(s) => s.read().await,
            Self::Mock(s) => s.read().await,
        }
    }
}
```

Same shape for `PumpDriver` and `Webcam`.

**Selection at startup** (probe-then-fallback):

1. If `MOESTUIN_MOCK_HW=1` → force `Mock`.
2. Else try to construct `Real` (open GPIO/I²C/`/dev/video0`). On success → `Real`.
3. On failure → log a `tracing::warn!` with the reason and fall back to `Mock`.

This means dev laptops, CI, and a Pi with a disconnected sensor all keep running; the health endpoint reports which variant is active per device so you can tell at a glance.

- `RealSensor` uses `rppal` for GPIO/I²C, compile-gated with `#[cfg(target_os = "linux")]`.
- `MockSensor` produces deterministic sine-wave values seeded from the wall clock so graphs look alive.

## Pumps — safety first

- Enforce a minimum interval between runs (default 10 min) and a max duration (default 120 s) in code, not just config.
- Every pump run writes a `pump_runs` row with `started_at`, `ended_at`, `requested_duration_s`, `actual_duration_s`, `trigger` (`manual|schedule`), `user_id`.
- Emergency stop: `POST /api/pumps/stop-all` forces all relays off and cancels scheduled tasks.
- On startup, drive all pump GPIOs to OFF before anything else.

## Webcam

- Capture with ffmpeg into HLS:
  ```bash
  ffmpeg -f v4l2 -i /dev/video0 -r 15 -s 1280x720 \
    -c:v libx264 -preset veryfast -g 30 \
    -f hls -hls_time 5 -hls_list_size 12 -hls_flags delete_segments+append_list \
    -strftime 1 -hls_segment_filename "seg-%Y%m%d-%H%M%S.ts" \
    live.m3u8
  ```
- Store segments under `/var/lib/moestuin/webcam/YYYY-MM-DD/`.
- Nightly cron: concat yesterday's segments into `YYYY-MM-DD.mp4`, delete originals, prune anything older than `WEBCAM_RETAIN_DAYS`.
- Serve live via the Caddy reverse proxy with `Cache-Control: no-store` on the `.m3u8`.

## Deploy

- `deploy/install.sh` copies the release binary to `/usr/local/bin/moestuin`, installs systemd units:
  - `moestuin.service` — the Axum server.
  - `moestuin-webcam.service` — the ffmpeg capture loop.
  - `moestuin-backup.timer` — nightly SQLite backup + webcam rollup.
- Grant the service user membership of `gpio`, `i2c`, `video` groups; don't run as root.

## Don't

- Don't toggle GPIO from a handler thread — go through the pump driver so the safety rules apply.
- Don't skip the startup-OFF step for pump pins.
- Don't check in `/dev/video0` test recordings or any photos.
