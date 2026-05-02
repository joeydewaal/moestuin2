import { defineConfig } from 'vitest/config';
import { sveltekit } from '@sveltejs/kit/vite';
import tailwindcss from '@tailwindcss/vite';

const backend = process.env.MOESTUIN_BACKEND ?? 'http://localhost:8080';

export default defineConfig({
	plugins: [tailwindcss(), sveltekit()],
	server: {
		proxy: {
			'/api': { target: backend, changeOrigin: false },
			'/auth': { target: backend, changeOrigin: false },
			// Only match the static-file subpaths (`/webcam/<date>/…`); the bare
			// `/webcam` page route belongs to SvelteKit and must not be proxied.
			'^/webcam/.+': { target: backend, changeOrigin: false }
		}
	},
	test: {
		expect: { requireAssertions: true },
		projects: [
			{
				extends: './vite.config.ts',
				test: {
					name: 'server',
					environment: 'node',
					include: ['src/**/*.{test,spec}.{js,ts}'],
					exclude: ['src/**/*.svelte.{test,spec}.{js,ts}']
				}
			}
		]
	}
});
