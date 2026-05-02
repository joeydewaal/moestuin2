import { test, expect } from '@playwright/test';

const me = { id: '00000000-0000-0000-0000-000000000001', email: 'fam@example.com', name: 'Fam' };

const live = { url: '/webcam/2026-05-01/live.m3u8', date: '2026-05-01' };
const archive = [
	{ date: '2026-04-30', url: '/webcam/2026-04-30/day.mp4' },
	{ date: '2026-04-29', url: '/webcam/2026-04-29/day.mp4' }
];
const emptyPlaylist = '#EXTM3U\n#EXT-X-VERSION:3\n#EXT-X-ENDLIST\n';

async function stubWebcamApis(page: import('@playwright/test').Page) {
	await page.route('**/auth/me', (route) => route.fulfill({ status: 200, json: me }));
	await page.route('**/api/webcam/live', (route) => route.fulfill({ status: 200, json: live }));
	await page.route('**/api/webcam/archive', (route) =>
		route.fulfill({ status: 200, json: archive })
	);
	// hls.js (or the native player) will fetch whatever URL we set as src.
	// Return a short, empty-on-purpose playlist so the player resolves quickly
	// instead of timing out network calls during the test.
	await page.route('**/webcam/**/*.m3u8', (route) =>
		route.fulfill({
			status: 200,
			headers: { 'content-type': 'application/vnd.apple.mpegurl' },
			body: emptyPlaylist
		})
	);
	await page.route('**/webcam/**/*.mp4', (route) =>
		route.fulfill({
			status: 200,
			headers: { 'content-type': 'video/mp4' },
			body: ''
		})
	);
}

test('webcam page redirects to /login when unauthenticated', async ({ page }) => {
	await page.route('**/auth/me', (route) => route.fulfill({ status: 401, body: '' }));
	await page.goto('/webcam');
	await page.waitForURL('**/login');
	await expect(page.getByTestId('google-signin')).toBeVisible();
});

test('webcam page renders live player and archive list', async ({ page }) => {
	await stubWebcamApis(page);

	await page.goto('/webcam');

	const player = page.getByTestId('webcam-player');
	await expect(player).toBeVisible();
	await expect(player).toHaveAttribute('data-src', live.url);

	const items = page.getByTestId('webcam-archive-item');
	await expect(items).toHaveCount(archive.length);
	await expect(items.first()).toHaveAttribute('data-date', archive[0].date);
	await expect(items.last()).toHaveAttribute('data-date', archive[1].date);
});

test('clicking an archive day swaps the player source, "Nu live" returns to live', async ({
	page
}) => {
	await stubWebcamApis(page);

	await page.goto('/webcam');

	const player = page.getByTestId('webcam-player');
	await expect(player).toHaveAttribute('data-src', live.url);

	// Pick the older day so we know the click swapped, not just stuck on first.
	await page.getByTestId('webcam-archive-item').last().click();
	await expect(player).toHaveAttribute('data-src', archive[1].url);

	const liveToggle = page.getByTestId('webcam-live-toggle');
	await expect(liveToggle).toBeVisible();
	await liveToggle.click();
	await expect(player).toHaveAttribute('data-src', live.url);
	await expect(liveToggle).toHaveCount(0);
});

test('webcam page shows offline placeholder when live URL is null', async ({ page }) => {
	await page.route('**/auth/me', (route) => route.fulfill({ status: 200, json: me }));
	await page.route('**/api/webcam/live', (route) =>
		route.fulfill({ status: 200, json: { url: null, date: null } })
	);
	await page.route('**/api/webcam/archive', (route) => route.fulfill({ status: 200, json: [] }));

	await page.goto('/webcam');

	await expect(page.getByTestId('webcam-offline')).toBeVisible();
});
