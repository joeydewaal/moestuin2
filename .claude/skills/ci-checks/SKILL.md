---
name: ci-checks
description: Commands and expectations for Moestuin CI — formatting, linting, and tests for both frontend and backend. Use before declaring a task done or when editing workflows.
---

# CI checks (Moestuin)

Run these locally before asking the user to review. CI enforces the same set.

## Backend (`server/`)

```bash
cargo fmt --all -- --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test
cargo insta review    # only if snapshot diffs appear
```

## Frontend (`web/`)

```bash
pnpm install --frozen-lockfile
pnpm format:check       # prettier
pnpm lint               # eslint
pnpm check              # svelte-check
pnpm test               # vitest
pnpm test:e2e           # playwright
```

## Workflows

`.github/workflows/`:
- `backend.yml` — matrix on stable Rust; caches `~/.cargo` and `target/`.
- `frontend.yml` — Node 20, pnpm cache, Playwright browsers cache.
- Both run on PR and on push to `main`. Required to pass before merge.

## When adding a feature

- New endpoint → add insta snapshot test.
- New UI feature → add Playwright spec under `web/tests/`.
- Touching hardware → ensure the mock driver path still compiles and the test suite runs off-Pi.

## Don't

- Don't commit with failing `fmt` or `clippy`. No `#[allow(...)]` without a comment explaining why.
- Don't skip Playwright by marking tests `.skip` — fix or delete.
- Don't regenerate insta snapshots blindly; review each diff.
