<script lang="ts">
	interface Props {
		data: {
			graphs: Promise<string[]>;
		};
	}

	let { data }: Props = $props();
</script>

<div class="animate-fade-in">
	<h1 class="mb-6 text-2xl font-bold text-surface-900 dark:text-white">Graphs</h1>

	{#await data.graphs}
		<div class="space-y-3">
			{#each Array(3) as _}
				<div class="skeleton h-12 w-full rounded-lg"></div>
			{/each}
		</div>
	{:then graphs}
		{#if graphs.length === 0}
			<div class="card text-center">
				<p class="text-surface-500 dark:text-surface-400">No graphs found</p>
			</div>
		{:else}
			<div class="grid gap-3">
				{#each graphs as graph, i}
					<a
						href={`graphs/${graph}`}
						class="card flex items-center justify-between transition-all hover:border-primary-300 hover:shadow-md dark:hover:border-primary-600"
						style="animation-delay: {i * 50}ms"
					>
						<div class="flex items-center gap-3">
							<div
								class="flex h-10 w-10 items-center justify-center rounded-lg bg-primary-100 text-primary-600 dark:bg-primary-900/50 dark:text-primary-400"
							>
								<svg class="h-5 w-5" fill="none" viewBox="0 0 24 24" stroke="currentColor">
									<path
										stroke-linecap="round"
										stroke-linejoin="round"
										stroke-width="2"
										d="M19 11H5m14 0a2 2 0 012 2v6a2 2 0 01-2 2H5a2 2 0 01-2-2v-6a2 2 0 012-2m14 0V9a2 2 0 00-2-2M5 11V9a2 2 0 012-2m0 0V5a2 2 0 012-2h6a2 2 0 012 2v2M7 7h10"
									/>
								</svg>
							</div>
							<span class="font-medium text-surface-900 dark:text-white">{graph}</span>
						</div>
						<svg
							class="h-5 w-5 text-surface-400"
							fill="none"
							viewBox="0 0 24 24"
							stroke="currentColor"
						>
							<path
								stroke-linecap="round"
								stroke-linejoin="round"
								stroke-width="2"
								d="M9 5l7 7-7 7"
							/>
						</svg>
					</a>
				{/each}
			</div>
		{/if}
	{:catch}
		<div class="card border-red-200 bg-red-50 dark:border-red-800 dark:bg-red-900/20">
			<p class="text-red-600 dark:text-red-400">Failed to load graphs</p>
		</div>
	{/await}
</div>
