<script lang="ts">
	import '../app.css';
	import { page } from '$app/stores';

	let { children } = $props();

	let darkMode = $state(false);

	function toggleDarkMode() {
		darkMode = !darkMode;
		if (darkMode) {
			document.documentElement.classList.add('dark');
		} else {
			document.documentElement.classList.remove('dark');
		}
	}

	const navItems = [
		{ href: '/', label: 'Graphs' },
		{ href: '/map', label: 'Map' },
		{ href: '/utils', label: 'Utils' }
	];
</script>

<div class="min-h-screen">
	<nav
		class="border-b border-surface-200 bg-white px-6 py-4 dark:border-surface-700 dark:bg-surface-800"
	>
		<div class="mx-auto flex max-w-7xl items-center justify-between">
			<div class="flex items-center gap-8">
				<a href="/" class="text-xl font-bold text-surface-900 dark:text-white"> Cytos </a>
				<div class="flex gap-1">
					{#each navItems as item}
						<a
							href={item.href}
							class="rounded-lg px-3 py-2 text-sm font-medium transition-colors
								{$page.url.pathname === item.href
								? 'bg-primary-50 text-primary-600 dark:bg-primary-900/30 dark:text-primary-400'
								: 'text-surface-600 hover:bg-surface-100 dark:text-surface-400 dark:hover:bg-surface-700'}"
						>
							{item.label}
						</a>
					{/each}
				</div>
			</div>
			<button
				onclick={toggleDarkMode}
				class="rounded-lg p-2 text-surface-500 hover:bg-surface-100 dark:text-surface-400 dark:hover:bg-surface-700"
				aria-label="Toggle dark mode"
			>
				{#if darkMode}
					<svg class="h-5 w-5" fill="none" viewBox="0 0 24 24" stroke="currentColor">
						<path
							stroke-linecap="round"
							stroke-linejoin="round"
							stroke-width="2"
							d="M12 3v1m0 16v1m9-9h-1M4 12H3m15.364 6.364l-.707-.707M6.343 6.343l-.707-.707m12.728 0l-.707.707M6.343 17.657l-.707.707M16 12a4 4 0 11-8 0 4 4 0 018 0z"
						/>
					</svg>
				{:else}
					<svg class="h-5 w-5" fill="none" viewBox="0 0 24 24" stroke="currentColor">
						<path
							stroke-linecap="round"
							stroke-linejoin="round"
							stroke-width="2"
							d="M20.354 15.354A9 9 0 018.646 3.646 9.003 9.003 0 0012 21a9.003 9.003 0 008.354-5.646z"
						/>
					</svg>
				{/if}
			</button>
		</div>
	</nav>

	<main class="mx-auto max-w-7xl px-6 py-8">
		{@render children()}
	</main>
</div>
