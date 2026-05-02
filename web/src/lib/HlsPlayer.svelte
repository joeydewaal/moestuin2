<script lang="ts">
	import Spinner from './Spinner.svelte';

	interface Props {
		src: string | null;
		/** Show muted/autoplay so live tile starts on its own without user gesture. */
		live?: boolean;
	}

	let { src, live = false }: Props = $props();

	let video = $state<HTMLVideoElement | null>(null);
	// Held as `unknown` so the cleanup closure can still call `.destroy()` after
	// dynamic import; typing it as the imported module's class would force eager
	// loading.
	let hls = $state<{ destroy(): void } | null>(null);
	let loadError = $state<string | null>(null);

	$effect(() => {
		const url = src;
		const el = video;
		if (!el || !url) {
			return;
		}
		loadError = null;

		// Native HLS (Safari, iOS): assign directly, no library.
		if (el.canPlayType('application/vnd.apple.mpegurl')) {
			el.src = url;
			return () => {
				el.removeAttribute('src');
				el.load();
			};
		}

		// Other browsers: lazy-load hls.js so the dashboard chunk stays slim.
		let cancelled = false;
		(async () => {
			const Hls = (await import('hls.js')).default;
			if (cancelled || !video) return;
			if (!Hls.isSupported()) {
				loadError = 'Deze browser ondersteunt geen HLS-video.';
				return;
			}
			const inst = new Hls({ liveDurationInfinity: live });
			inst.loadSource(url);
			inst.attachMedia(video);
			inst.on(Hls.Events.ERROR, (_, data) => {
				if (data.fatal) {
					loadError = 'Video kon niet worden geladen.';
				}
			});
			hls = inst;
		})();

		return () => {
			cancelled = true;
			if (hls) {
				hls.destroy();
				hls = null;
			}
			if (el) {
				el.removeAttribute('src');
				el.load();
			}
		};
	});
</script>

<div
	class="relative aspect-video w-full overflow-hidden rounded-md bg-black"
	data-testid="webcam-player"
	data-src={src ?? ''}
>
	{#if !src}
		<div class="absolute inset-0 flex items-center justify-center text-surface-300">
			<span data-testid="webcam-offline">Camera staat uit.</span>
		</div>
	{:else if loadError}
		<div
			class="absolute inset-0 flex items-center justify-center text-sm text-surface-200 px-4 text-center"
			data-testid="webcam-error"
		>
			{loadError}
		</div>
	{/if}

	<video
		bind:this={video}
		class="w-full h-full"
		controls
		playsinline
		autoplay={live}
		muted={live}
		data-testid="webcam-video"
	>
		<track kind="captions" />
	</video>

	{#if src && !loadError && !hls && !video?.canPlayType('application/vnd.apple.mpegurl')}
		<div class="absolute inset-0 flex items-center justify-center bg-black/40">
			<Spinner />
		</div>
	{/if}
</div>
