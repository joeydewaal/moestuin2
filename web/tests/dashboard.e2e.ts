import { test, expect } from '@playwright/test';

const me = { id: '00000000-0000-0000-0000-000000000001', email: 'fam@example.com', name: 'Fam' };

function reading(secondsAgo: number, temp: number, humidity: number, moisture: number) {
	return {
		id: `r-${secondsAgo}`,
		taken_at: new Date(Date.now() - secondsAgo * 1000).toISOString(),
		temp_c: temp,
		humidity,
		moisture
	};
}

test('dashboard renders graph and live SSE updates latest value', async ({ page }) => {
	await page.route('**/auth/me', (route) => route.fulfill({ status: 200, json: me }));
	await page.route('**/api/readings*', (route) => {
		if (route.request().url().includes('/stream')) return;
		route.fulfill({
			status: 200,
			json: [reading(120, 20.5, 60, 0.4), reading(60, 21.0, 61, 0.42)]
		});
	});
	await page.route('**/api/readings/stream', async (route) => {
		const live = reading(0, 22.5, 65, 0.5);
		const body = `: open\n\ndata: ${JSON.stringify(live)}\n\n`;
		await route.fulfill({
			status: 200,
			headers: { 'content-type': 'text/event-stream', 'cache-control': 'no-cache' },
			body
		});
	});

	await page.goto('/');
	await expect(page.getByTestId('sensor-graph-temp_c')).toBeVisible();
	await expect(page.getByTestId('latest-temp_c')).toHaveText('22.5°C');
	await expect(page.getByTestId('latest-humidity')).toHaveText('65.0%');
});

test('theme toggle persists to localStorage', async ({ page }) => {
	await page.route('**/auth/me', (route) => route.fulfill({ status: 200, json: me }));
	await page.route('**/api/readings*', (route) => {
		if (route.request().url().includes('/stream')) return;
		route.fulfill({ status: 200, json: [] });
	});
	await page.route('**/api/readings/stream', (route) =>
		route.fulfill({
			status: 200,
			headers: { 'content-type': 'text/event-stream' },
			body: ': open\n\n'
		})
	);

	await page.goto('/');
	const toggle = page.getByTestId('theme-toggle');
	await expect(toggle).toBeVisible();
	await toggle.click();
	const stored = await page.evaluate(() => localStorage.getItem('moestuin:theme'));
	expect(stored === 'dark' || stored === 'light').toBe(true);
});
