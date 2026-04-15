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

## Deploy (Pi, Docker Compose)

```bash
cp deploy/moestuin.env.example deploy/moestuin.env
# edit deploy/moestuin.env, set MOESTUIN_SITE, OIDC creds, allowed emails
docker compose up -d --build
```

Nightly backup (host cron):

```cron
15 3 * * *  cd /srv/moestuin && docker compose run --rm backup
```

Adjust `group_add` in `docker-compose.yml` to match the host's `gpio` and `i2c` GIDs (`getent group gpio i2c`).
