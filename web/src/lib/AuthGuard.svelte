<script lang="ts">
	import { goto } from '$app/navigation';
	import { resolve } from '$app/paths';
	import { sessionQuery } from './session';
	import Spinner from './Spinner.svelte';

	let { children } = $props();
	const session = sessionQuery();

	$effect(() => {
		if (!session.isLoading && session.data === null) {
			goto(resolve('/login'), { replaceState: true });
		}
	});
</script>

{#if session.isLoading}
	<Spinner />
{:else if session.data}
	{@render children()}
{/if}
