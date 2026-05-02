<script lang="ts">
	import { page } from '$app/state';
	import { resolve } from '$app/paths';
	import AuthGuard from '$lib/AuthGuard.svelte';
	import ThemeToggle from '$lib/ThemeToggle.svelte';
	import { sessionQuery } from '$lib/session';
	import { initTheme, type Theme } from '$lib/theme';

	let { children } = $props();
	const session = sessionQuery();

	let theme = $state<Theme>('light');

	$effect(() => {
		theme = initTheme();
	});

	const isActive = (href: string) =>
		href === resolve('/') ? page.url.pathname === '/' : page.url.pathname.startsWith(href);
</script>

<AuthGuard>
	<div class="min-h-screen bg-surface-50 dark:bg-surface-950">
		<header
			class="border-b border-surface-200-700 bg-surface-50/80 dark:bg-surface-950/80 backdrop-blur-sm sticky top-0 z-10"
		>
			<div class="mx-auto max-w-5xl px-4 sm:px-6 h-14 flex items-center justify-between gap-4">
				<div class="flex items-center gap-2">
					<span class="text-xl" aria-hidden="true">🌱</span>
					<span class="font-semibold text-surface-900 dark:text-surface-50">Mijn Moestuin</span>
				</div>
				<div class="flex items-center gap-3">
					{#if session.data}
						<span
							class="hidden sm:block text-sm text-surface-500-400 truncate max-w-48"
							data-testid="whoami"
						>
							{session.data.email}
						</span>
					{/if}
					<ThemeToggle bind:theme />
					<form method="get" action="/auth/logout">
						<button class="btn preset-tonal-surface text-sm py-1.5 px-3 rounded-md" type="submit">
							Uitloggen
						</button>
					</form>
				</div>
			</div>
			<nav
				class="mx-auto max-w-5xl px-4 sm:px-6 flex items-center gap-1 text-sm"
				data-testid="primary-nav"
			>
				<a
					href={resolve('/')}
					class={[
						'px-3 py-2 -mb-px border-b-2 transition-colors',
						isActive(resolve('/'))
							? 'border-primary-500 text-surface-900 dark:text-surface-50 font-medium'
							: 'border-transparent text-surface-500-400 hover:text-surface-900 dark:hover:text-surface-50'
					].join(' ')}
					data-testid="nav-dashboard"
					aria-current={isActive(resolve('/')) ? 'page' : undefined}
				>
					Dashboard
				</a>
				<a
					href={resolve('/(app)/webcam')}
					class={[
						'px-3 py-2 -mb-px border-b-2 transition-colors',
						isActive(resolve('/(app)/webcam'))
							? 'border-primary-500 text-surface-900 dark:text-surface-50 font-medium'
							: 'border-transparent text-surface-500-400 hover:text-surface-900 dark:hover:text-surface-50'
					].join(' ')}
					data-testid="nav-webcam"
					aria-current={isActive(resolve('/(app)/webcam')) ? 'page' : undefined}
				>
					Webcam
				</a>
			</nav>
		</header>

		<main class="mx-auto max-w-5xl px-4 sm:px-6 py-8">
			{@render children()}
		</main>
	</div>
</AuthGuard>
