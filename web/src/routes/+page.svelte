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
	<main class="mx-auto max-w-4xl p-6">
		<header class="flex items-center justify-between">
			<div>
				<h1 class="text-2xl font-semibold">Moestuin</h1>
				{#if session.data}
					<p class="text-sm opacity-80" data-testid="whoami">
						Signed in as {session.data.email}
					</p>
				{/if}
			</div>
			<ThemeToggle bind:theme />
		</header>

		<section class="mt-6 space-y-4" data-testid="dashboard">
			{#if readings.isLoading}
				<div data-testid="readings-loading"><Spinner /></div>
			{:else if readings.isError}
				<p class="text-error-500">Failed to load readings</p>
			{:else if readings.data}
				<SensorGraph
					readings={readings.data}
					field="temp_c"
					label="Temperature"
					color="#ef4444"
					unit="°C"
				/>
				<SensorGraph
					readings={readings.data}
					field="humidity"
					label="Humidity"
					color="#3b82f6"
					unit="%"
				/>
				<SensorGraph
					readings={readings.data}
					field="moisture"
					label="Soil moisture"
					color="#10b981"
				/>
			{/if}
		</section>

		<form method="get" action="/auth/logout" class="mt-8">
			<button class="btn preset-tonal" type="submit">Sign out</button>
		</form>
	</main>
</AuthGuard>
