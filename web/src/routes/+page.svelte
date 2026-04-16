<script lang="ts">
	import AuthGuard from '$lib/AuthGuard.svelte';
	import { sessionQuery } from '$lib/session';

	const session = sessionQuery();
</script>

<AuthGuard>
	<main class="mx-auto max-w-3xl p-6">
		<h1 class="text-2xl font-semibold">Moestuin</h1>
		{#if session.data}
			<p class="mt-2 text-sm opacity-80" data-testid="whoami">
				Signed in as {session.data.email}
			</p>
		{/if}
		<p class="mt-6 opacity-70">Sensors, crops, pumps, webcam — coming soon.</p>
		<form method="get" action="/auth/logout" class="mt-8">
			<button class="btn preset-tonal" type="submit">Sign out</button>
		</form>
	</main>
</AuthGuard>
