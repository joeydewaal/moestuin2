import { test, expect } from '@playwright/test';

test('unauthenticated visitor is redirected to /login', async ({ page }) => {
	await page.route('**/auth/me', (route) => route.fulfill({ status: 401, body: '' }));

	await page.goto('/');
	await page.waitForURL('**/login');
	await expect(page.getByTestId('google-signin')).toBeVisible();
});

test('login page renders Google sign-in button', async ({ page }) => {
	await page.goto('/login');
	const btn = page.getByTestId('google-signin');
	await expect(btn).toBeVisible();
	await expect(btn).toHaveAttribute('href', '/auth/login');
});
