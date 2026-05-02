<script lang="ts">
	import HlsPlayer from '$lib/HlsPlayer.svelte';
	import Spinner from '$lib/Spinner.svelte';
	import { archiveQuery, liveQuery } from '$lib/webcam';

	const live = liveQuery();
	const archive = archiveQuery();

	/** When non-null, the user is watching an archived day instead of the live feed. */
	let selectedUrl = $state<string | null>(null);
	let selectedDate = $state<string | null>(null);

	const dateFormatter = new Intl.DateTimeFormat('nl-NL', {
		weekday: 'long',
		day: 'numeric',
		month: 'long'
	});

	function formatDate(ymd: string): string {
		// Backend returns plain YYYY-MM-DD strings; treat as local-day labels.
		const [y, m, d] = ymd.split('-').map(Number);
		const dt = new Date(y, m - 1, d);
		return dateFormatter.format(dt);
	}

	function selectArchive(entry: { date: string; url: string }) {
		selectedUrl = entry.url;
		selectedDate = entry.date;
	}

	function goLive() {
		selectedUrl = null;
		selectedDate = null;
	}

	const playerSrc = $derived(selectedUrl ?? live.data?.url ?? null);
	const isLiveMode = $derived(selectedUrl === null);
</script>

<div class="mb-6 flex items-end justify-between gap-3">
	<div>
		<h1 class="text-2xl font-semibold text-surface-900 dark:text-surface-50">Live camera</h1>
		<p class="text-sm text-surface-500-400 mt-1">
			{#if isLiveMode}
				{#if live.data?.date}
					Beelden van vandaag, {formatDate(live.data.date)}.
				{:else}
					Geen live beelden beschikbaar.
				{/if}
			{:else if selectedDate}
				Archief van {formatDate(selectedDate)}.
			{/if}
		</p>
	</div>
	{#if !isLiveMode}
		<button
			class="btn preset-tonal-surface text-sm py-1.5 px-3 rounded-md"
			data-testid="webcam-live-toggle"
			onclick={goLive}
		>
			Nu live
		</button>
	{/if}
</div>

<section class="space-y-6">
	{#if live.isLoading}
		<div data-testid="webcam-loading"><Spinner /></div>
	{:else if live.isError}
		<div class="card preset-tonal-error p-4 text-sm" data-testid="webcam-load-error">
			Camera kon niet worden geladen.
			<button class="underline" onclick={() => live.refetch()}>Opnieuw proberen</button>
		</div>
	{:else}
		<HlsPlayer src={playerSrc} live={isLiveMode} />
	{/if}

	<aside data-testid="webcam-archive">
		<h2 class="text-lg font-semibold text-surface-900 dark:text-surface-50 mb-3">Archief</h2>
		{#if archive.isLoading}
			<Spinner />
		{:else if archive.isError}
			<div class="card preset-tonal-error p-4 text-sm">
				Archief kon niet worden geladen.
				<button class="underline" onclick={() => archive.refetch()}>Opnieuw proberen</button>
			</div>
		{:else if archive.data && archive.data.length > 0}
			<ul class="grid gap-2 sm:grid-cols-2">
				{#each archive.data as entry (entry.date)}
					<li>
						<button
							type="button"
							class="w-full text-left card preset-filled-surface-100-800 p-3 rounded-md hover:opacity-90 transition-opacity {selectedUrl ===
							entry.url
								? 'ring-2 ring-primary-500'
								: ''}"
							data-testid="webcam-archive-item"
							data-date={entry.date}
							onclick={() => selectArchive(entry)}
						>
							<span class="block font-medium text-surface-900 dark:text-surface-50">
								{formatDate(entry.date)}
							</span>
							<span class="block text-xs text-surface-500-400">{entry.date}</span>
						</button>
					</li>
				{/each}
			</ul>
		{:else}
			<p class="text-sm text-surface-500-400">Nog geen archief beschikbaar.</p>
		{/if}
	</aside>
</section>
