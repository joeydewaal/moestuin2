#!/usr/bin/env bash
# Install Moestuin onto a Raspberry Pi. Idempotent; re-run after code changes.
set -euo pipefail

REPO_ROOT="$(cd "$(dirname "$0")/.." && pwd)"
PREFIX=/usr/local/bin
STATE=/var/lib/moestuin
ETC=/etc/moestuin

sudo id -u moestuin >/dev/null 2>&1 || sudo useradd --system --home "$STATE" --shell /usr/sbin/nologin moestuin
sudo usermod -aG gpio,i2c,video moestuin || true

sudo install -d -o moestuin -g moestuin "$STATE" "$STATE/webcam" "$STATE/photos" "$STATE/backups"
sudo install -d "$ETC"

echo "==> building backend"
(cd "$REPO_ROOT/server" && cargo build --release)
sudo install -m 0755 "$REPO_ROOT/server/target/release/server" "$PREFIX/moestuin"

echo "==> building frontend"
(cd "$REPO_ROOT/web" && pnpm install --frozen-lockfile && pnpm build)
sudo rsync -a --delete "$REPO_ROOT/web/build/" "$STATE/web/"

echo "==> installing units"
sudo install -m 0644 "$REPO_ROOT/deploy"/moestuin*.service /etc/systemd/system/
sudo install -m 0644 "$REPO_ROOT/deploy/moestuin-backup.timer" /etc/systemd/system/
sudo install -m 0755 "$REPO_ROOT/deploy/moestuin-webcam.sh" "$PREFIX/moestuin-webcam.sh"
sudo install -m 0755 "$REPO_ROOT/deploy/moestuin-backup.sh" "$PREFIX/moestuin-backup.sh"

if [ ! -f "$ETC/moestuin.env" ]; then
	sudo install -m 0640 -o root -g moestuin "$REPO_ROOT/deploy/moestuin.env.example" "$ETC/moestuin.env"
	echo "==> wrote $ETC/moestuin.env — edit before starting"
fi

sudo systemctl daemon-reload
sudo systemctl enable --now moestuin.service moestuin-webcam.service moestuin-backup.timer

echo "==> done"
