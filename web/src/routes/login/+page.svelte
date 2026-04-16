<script lang="ts">
	import { page } from '$app/state';

	const errorMessages: Record<string, string> = {
		no_email: 'Je Google-account heeft geen e-mailadres teruggegeven.',
		unverified: 'Je Google-e-mailadres is niet geverifieerd.',
		forbidden: 'Dit e-mailadres staat niet op de toegestane lijst.'
	};

	const err = $derived(page.url.searchParams.get('error'));
	const errMsg = $derived(err ? (errorMessages[err] ?? 'Inloggen mislukt.') : null);
</script>

<svelte:head>
	<title>Inloggen — Mijn Moestuin</title>
</svelte:head>

<div class="min-h-screen bg-surface-50 dark:bg-surface-950 flex items-center justify-center p-4">
	<div class="w-full max-w-sm">
		<div class="text-center mb-8">
			<div
				class="inline-flex items-center justify-center w-16 h-16 rounded-2xl bg-primary-500/10 mb-4"
			>
				<span class="text-3xl" aria-hidden="true">🌱</span>
			</div>
			<h1 class="text-2xl font-semibold text-surface-900 dark:text-surface-50">Mijn Moestuin</h1>
			<p class="mt-1 text-sm text-surface-500-400">Familietuin tracker</p>
		</div>

		<div class="card preset-filled-surface-100-800 shadow-md p-6 space-y-4">
			{#if errMsg}
				<div class="card preset-tonal-error p-3 text-sm" role="alert">
					{errMsg}
				</div>
			{/if}

			<p class="text-sm text-surface-600-300 text-center">
				Log in met je familie Google-account om verder te gaan.
			</p>

			<!-- eslint-disable-next-line svelte/no-navigation-without-resolve -- backend route -->
			<a
				href="/auth/login"
				class="btn preset-filled w-full justify-center gap-2 py-2.5"
				data-testid="google-signin"
			>
				<svg
					width="18"
					height="18"
					viewBox="0 0 48 48"
					aria-hidden="true"
					xmlns="http://www.w3.org/2000/svg"
				>
					<path
						fill="#EA4335"
						d="M24 9.5c3.54 0 6.71 1.22 9.21 3.6l6.85-6.85C35.9 2.38 30.47 0 24 0 14.62 0 6.51 5.38 2.56 13.22l7.98 6.19C12.43 13.72 17.74 9.5 24 9.5z"
					/>
					<path
						fill="#4285F4"
						d="M46.98 24.55c0-1.57-.15-3.09-.38-4.55H24v9.02h12.94c-.58 2.96-2.26 5.48-4.78 7.18l7.73 6c4.51-4.18 7.09-10.36 7.09-17.65z"
					/>
					<path
						fill="#FBBC05"
						d="M10.53 28.59c-.48-1.45-.76-2.99-.76-4.59l-7.98-6.19C.92 16.46 0 20.12 0 24c0 3.88.92 7.54 2.56 10.78l7.97-6.19z"
					/>
					<path
						fill="#34A853"
						d="M24 48c6.48 0 11.93-2.13 15.89-5.81l-7.73-6c-2.15 1.45-4.92 2.3-8.16 2.3-6.26 0-11.57-4.22-13.47-9.91l-7.98 6.19C6.51 42.62 14.62 48 24 48z"
					/>
				</svg>
				Inloggen met Google
			</a>

			{#if import.meta.env.DEV}
				<hr class="border-surface-200-700" />
				<div class="space-y-2">
					<p class="text-xs text-surface-400 text-center uppercase tracking-wider">
						Ontwikkelomgeving
					</p>
					<!-- eslint-disable-next-line svelte/no-navigation-without-resolve -- backend route via Vite proxy -->
					<form method="get" action="/auth/dev-login" class="flex gap-2">
						<input
							type="email"
							name="email"
							placeholder="e-mailadres"
							required
							class="input flex-1 text-sm"
							data-testid="dev-email"
						/>
						<button
							type="submit"
							class="btn preset-tonal-surface text-sm px-3"
							data-testid="dev-login"
						>
							Inloggen
						</button>
					</form>
				</div>
			{/if}
		</div>
	</div>
</div>
