<script lang="ts">
	import type { Reading } from './api';

	interface Props {
		readings: Reading[];
		field: 'temp_c' | 'humidity' | 'moisture';
		label: string;
		color: string;
		unit?: string;
	}
	let { readings, field, label, color, unit = '' }: Props = $props();

	const width = 600;
	const height = 160;
	const padding = { top: 10, right: 10, bottom: 20, left: 36 };

	const points = $derived.by(() => {
		if (readings.length === 0) return [] as { x: number; y: number; v: number; t: number }[];
		const times = readings.map((r) => new Date(r.taken_at).getTime());
		const vals = readings.map((r) => r[field]);
		const tMin = Math.min(...times);
		const tMax = Math.max(...times);
		const vMin = Math.min(...vals);
		const vMax = Math.max(...vals);
		const tSpan = tMax - tMin || 1;
		const vSpan = vMax - vMin || 1;
		const innerW = width - padding.left - padding.right;
		const innerH = height - padding.top - padding.bottom;
		return readings.map((r, i) => ({
			x: padding.left + ((times[i] - tMin) / tSpan) * innerW,
			y: padding.top + innerH - ((vals[i] - vMin) / vSpan) * innerH,
			v: vals[i],
			t: times[i]
		}));
	});

	const path = $derived(points.map((p, i) => `${i === 0 ? 'M' : 'L'}${p.x},${p.y}`).join(' '));
	const latest = $derived(readings.at(-1)?.[field]);
</script>

<section class="card preset-filled-surface-100-900 p-4" data-testid="sensor-graph-{field}">
	<header class="flex items-baseline justify-between">
		<h2 class="text-lg font-semibold">{label}</h2>
		<span class="text-xl font-mono" data-testid="latest-{field}">
			{latest !== undefined ? `${latest.toFixed(2)}${unit}` : '—'}
		</span>
	</header>
	{#if points.length === 0}
		<p class="py-8 text-center opacity-60">No data yet</p>
	{:else}
		<svg viewBox="0 0 {width} {height}" class="w-full" role="img" aria-label={label}>
			<path d={path} fill="none" stroke={color} stroke-width="2" />
			{#each points as p (p.t)}
				<circle cx={p.x} cy={p.y} r="1.5" fill={color} />
			{/each}
		</svg>
	{/if}
</section>
