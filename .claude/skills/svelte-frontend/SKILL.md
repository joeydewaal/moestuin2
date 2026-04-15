---
name: svelte-frontend
description: Patterns for the Moestuin Svelte 5 PWA — runes, TanStack Query, Skeleton UI, SSE, Playwright tests. Use whenever editing files under web/.
---

# Svelte frontend (Moestuin)

Invoke this skill when working in `web/`.

## Stack reminders

- **Svelte 5 runes** only. No `$:` reactive statements, no stores from `svelte/store` unless interop is required. Use `$state`, `$derived`, `$effect`, `$props`.
- **TanStack Query** (`@tanstack/svelte-query`) is the single source of truth for server data. No ad-hoc `fetch` inside components.
- **Skeleton UI** v3 for components and theming. Prefer Skeleton primitives over hand-rolled HTML.
- **PWA**: don't bypass the service worker cache; additions to the precache list go through `vite.config.ts`.

## Data fetching

```ts
// web/src/lib/queries/readings.ts
import { createQuery } from '@tanstack/svelte-query';

export const readingsQuery = (range: Range) =>
  createQuery({
    queryKey: ['readings', range],
    queryFn: () => api.get('/api/readings', { params: range }),
    staleTime: 15_000,
  });
```

- Every query must have a stable `queryKey`.
- Every loading state must render a Skeleton `ProgressRing` or `Spinner` — never a blank pane.
- On error, show Skeleton toast via `getToastStore()` and a retry button.

## SSE → query cache

Live sensor updates come through `/api/readings/stream`. Pattern:

```ts
const client = useQueryClient();
$effect(() => {
  const es = new EventSource('/api/readings/stream');
  es.addEventListener('reading', (e) => {
    const reading = JSON.parse(e.data);
    client.setQueryData(['readings', 'latest'], (prev) => [...(prev ?? []), reading]);
  });
  return () => es.close();
});
```

Keep SSE handlers inside a single root-level component so we don't open multiple connections.

## Auth

- `useSession` query hits `/auth/me`. If 401, redirect to `/login`.
- Login page is the only route that renders without a session; everything else is wrapped by `<AuthGuard>`.

## Theming

- Light/dark via Skeleton's `data-theme` on `<html>`. Persist choice in `localStorage` under `moestuin:theme`.
- Respect `prefers-color-scheme` on first load.

## Testing

- Unit: Vitest, colocate `*.test.ts` with source.
- UI: Playwright specs under `web/tests/`. Every new feature ships at least one.
- Run locally: `pnpm test`, `pnpm test:e2e`.
- Mock the backend with MSW (`web/src/mocks/`) in Playwright to avoid hitting a live server.

## Don't

- Don't introduce another state lib (no Zustand, no Pinia-ish wrappers).
- Don't fetch in `onMount` — use TanStack Query.
- Don't hardcode colors; use Skeleton theme tokens.
