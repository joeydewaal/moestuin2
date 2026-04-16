<script lang="ts">
	import { applyTheme, type Theme } from './theme';

	let { theme = $bindable('light' as Theme) } = $props();

	function toggle() {
		theme = theme === 'dark' ? 'light' : 'dark';
		applyTheme(theme);
	}
</script>

<button
	type="button"
	class="btn preset-tonal"
	data-testid="theme-toggle"
	aria-label="Toggle theme"
	onclick={toggle}
>
	{theme === 'dark' ? 'Light' : 'Dark'} mode
</button>
