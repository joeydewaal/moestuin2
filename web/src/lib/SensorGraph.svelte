<script lang="ts">
	import { LineChart } from 'layerchart';
	import type { Reading } from './api';

	interface Props {
		readings: Reading[];
		field: 'temp_c' | 'humidity' | 'moisture';
		label: string;
		color: string;
		unit?: string;
		icon?: string;
	}
	let { readings, field, label, color, unit = '', icon = '' }: Props = $props();

	const latest = $derived(readings.at(-1)?.[field]);

	const trend = $derived.by(() => {
		if (readings.length < 2) return null;
		const prev = readings.at(-2)![field];
		const curr = readings.at(-1)![field];
		return curr > prev ? 'up' : curr < prev ? 'down' : 'flat';
	});
</script>

<div
	class="card preset-filled-surface-100-800 overflow-hidden shadow-sm"
	data-testid="sensor-graph-{field}"
>
	<div class="h-1 w-full" style="background-color: {color};"></div>
	<div class="p-4">
		<div class="flex items-start justify-between gap-4">
			<div>
				<p class="text-sm font-medium text-surface-600-300 uppercase tracking-wider">
					{#if icon}{icon}{/if}
					{label}
				</p>
				<p class="mt-1 text-3xl font-semibold tabular-nums" data-testid="latest-{field}">
					{latest !== undefined ? `${latest.toFixed(1)}` : '—'}<span
						class="text-lg ml-0.5 text-surface-500-400">{unit}</span
					>
				</p>
			</div>
			{#if trend}
				<span class="mt-1 text-lg" aria-label="trend {trend}">
					{trend === 'up' ? '↑' : trend === 'down' ? '↓' : '→'}
				</span>
			{/if}
		</div>

		{#if readings.length === 0}
			<p class="mt-4 py-6 text-center text-sm text-surface-400">No data yet</p>
		{:else}
			<div class="mt-3 h-32">
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
	</div>
</div>
