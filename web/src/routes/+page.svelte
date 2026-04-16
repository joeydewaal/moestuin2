<script lang="ts">
	import AuthGuard from '$lib/AuthGuard.svelte';
	import SensorGraph from '$lib/SensorGraph.svelte';
	import Spinner from '$lib/Spinner.svelte';
	import ThemeToggle from '$lib/ThemeToggle.svelte';
	import { sessionQuery } from '$lib/session';
	import { readingsQuery, useReadingsStream } from '$lib/readings';
	import { initTheme, type Theme } from '$lib/theme';

	const session = sessionQuery();
	const readings = readingsQuery();

	let theme = $state<Theme>('light');

	$effect(() => {
		theme = initTheme();
	});

	$effect(() => {
		const unsub = useReadingsStream();
		return () => unsub();
	});
</script>

<AuthGuard>
	<div class="min-h-screen bg-surface-50 dark:bg-surface-950">
		<!-- Top bar -->
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
		</header>

		<!-- Main content -->
		<main class="mx-auto max-w-5xl px-4 sm:px-6 py-8">
			<div class="mb-6">
				<h1 class="text-2xl font-semibold text-surface-900 dark:text-surface-50">Dashboard</h1>
				<p class="text-sm text-surface-500-400 mt-1">Live sensormetingen van de moestuin</p>
			</div>

			<section data-testid="dashboard">
				{#if readings.isLoading}
					<div data-testid="readings-loading">
						<Spinner />
					</div>
				{:else if readings.isError}
					<div class="card preset-tonal-error p-4 text-sm">
						Metingen konden niet worden geladen. <button
							class="underline"
							onclick={() => readings.refetch()}>Opnieuw proberen</button
						>
					</div>
				{:else if readings.data}
					<div class="grid gap-4 sm:grid-cols-3">
						<SensorGraph
							readings={readings.data}
							field="temp_c"
							label="Temperatuur"
							color="oklch(62% 0.2 25)"
							unit="°C"
							icon="🌡️"
						/>
						<SensorGraph
							readings={readings.data}
							field="humidity"
							label="Luchtvochtigheid"
							color="oklch(60% 0.15 240)"
							unit="%"
							icon="💧"
						/>
						<SensorGraph
							readings={readings.data}
							field="moisture"
							label="Bodemvochtigheid"
							color="oklch(55% 0.13 145)"
							unit=""
							icon="🌱"
						/>
					</div>
				{/if}
			</section>
		</main>
	</div>
</AuthGuard>
