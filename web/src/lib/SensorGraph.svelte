<script lang="ts">
	import { LineChart } from 'layerchart';
	import type { Reading } from './api';

	interface Props {
		readings: Reading[];
		field: 'temp_c' | 'humidity' | 'moisture';
		label: string;
		color: string;
		unit?: string;
	}
	let { readings, field, label, color, unit = '' }: Props = $props();

	const latest = $derived(readings.at(-1)?.[field]);
</script>

<section class="card preset-filled-surface-100-900 p-4" data-testid="sensor-graph-{field}">
	<header class="flex items-baseline justify-between mb-2">
		<h2 class="text-lg font-semibold">{label}</h2>
		<span class="text-xl font-mono" data-testid="latest-{field}">
			{latest !== undefined ? `${latest.toFixed(2)}${unit}` : '—'}
		</span>
	</header>
	{#if readings.length === 0}
		<p class="py-8 text-center opacity-60">No data yet</p>
	{:else}
		<div class="h-40">
			<LineChart
				data={readings}
				x={(d: Reading) => new Date(d.taken_at)}
				y={(d: Reading) => d[field]}
				series={[{ key: field, value: (d: Reading) => d[field], color }]}
				points={false}
				grid
			/>
		</div>
	{/if}
</section>
