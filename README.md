# Moestuin

Self-hosted allotment tracker running on a Raspberry Pi. Live sensor graphs, webcam feed and archive, crop log with photos, scheduled water pumps. Family-only, Google OIDC.

See [`CLAUDE.md`](./CLAUDE.md) for architecture and [`ROADMAP.md`](./ROADMAP.md) for milestones.

## Dev

```bash
# backend
cd server && MOESTUIN_MOCK_HW=1 cargo run

# frontend
cd web && pnpm install && pnpm dev
```

## Deploy (Pi)

```bash
./deploy/install.sh
```

Edit `/etc/moestuin/moestuin.env` afterwards, then `sudo systemctl restart moestuin`.
