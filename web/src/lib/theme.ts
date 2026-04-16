import { browser } from '$app/environment';

export type Theme = 'light' | 'dark';
const STORAGE_KEY = 'moestuin-theme';

export function initTheme(): Theme {
	if (!browser) return 'light';
	const stored = localStorage.getItem(STORAGE_KEY) as Theme | null;
	const prefersDark = window.matchMedia('(prefers-color-scheme: dark)').matches;
	const theme: Theme = stored ?? (prefersDark ? 'dark' : 'light');
	applyTheme(theme);
	return theme;
}

export function applyTheme(theme: Theme) {
	if (!browser) return;
	document.documentElement.dataset.mode = theme;
	document.documentElement.classList.toggle('dark', theme === 'dark');
	localStorage.setItem(STORAGE_KEY, theme);
}
