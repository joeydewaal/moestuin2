<script lang="ts">
	import '../app.css';
	import favicon from '$lib/assets/favicon.svg';
	import { QueryClient, QueryClientProvider } from '@tanstack/svelte-query';

	let { children } = $props();
	const client = new QueryClient({
		defaultOptions: {
			queries: { retry: false, refetchOnWindowFocus: false }
		}
	});
</script>

<svelte:head>
	<link rel="icon" href={favicon} />
	<title>Moestuin</title>
</svelte:head>

<QueryClientProvider {client}>
	{@render children()}
</QueryClientProvider>
