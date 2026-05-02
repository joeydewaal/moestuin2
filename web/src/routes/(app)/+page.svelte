<script lang="ts">
	import SensorGraph from '$lib/SensorGraph.svelte';
	import Spinner from '$lib/Spinner.svelte';
	import { readingsQuery, useReadingsStream } from '$lib/readings';

	const readings = readingsQuery();

	$effect(() => {
		const unsub = useReadingsStream();
		return () => unsub();
	});
</script>

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
