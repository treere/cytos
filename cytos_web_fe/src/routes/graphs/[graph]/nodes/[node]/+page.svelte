<script lang="ts">
	let { data } = $props();
</script>

<div class="animate-fade-in">
	<nav class="mb-6 flex flex-wrap items-center gap-2 text-sm">
		<a
			href="/"
			class="text-surface-500 hover:text-primary-600 dark:text-surface-400 dark:hover:text-primary-400"
			>root</a
		>
		<span class="text-surface-300 dark:text-surface-600">/</span>
		<a
			href={`/graphs/${data.graph}`}
			class="text-surface-500 hover:text-primary-600 dark:text-surface-400 dark:hover:text-primary-400"
			>{data.graph}</a
		>
		<span class="text-surface-300 dark:text-surface-600">/</span>
		<span class="font-mono font-medium text-surface-900 dark:text-white">{data.node}</span>
	</nav>

	<div class="grid gap-6 md:grid-cols-2">
		<div class="card">
			<h2 class="mb-4 text-lg font-semibold text-surface-900 dark:text-white">Inputs</h2>

			{#await data.inputs}
				<div class="space-y-2">
					{#each Array(3) as _}
						<div class="skeleton h-8 w-full rounded"></div>
					{/each}
				</div>
			{:then inputs}
				{#if inputs.length === 0}
					<p class="text-surface-500 dark:text-surface-400">No inputs</p>
				{:else}
					<ul class="space-y-1">
						{#each inputs as input}
							<li>
								<a
									href={`/graphs/${data.graph}/nodes/${data.node}/params/${input}`}
									class="flex items-center gap-2 rounded-lg px-3 py-2 transition-colors hover:bg-surface-100 dark:hover:bg-surface-700"
								>
									<svg
										class="h-4 w-4 text-surface-400"
										fill="none"
										viewBox="0 0 24 24"
										stroke="currentColor"
									>
										<path
											stroke-linecap="round"
											stroke-linejoin="round"
											stroke-width="2"
											d="M19 11H5m14 0a2 2 0 012 2v6a2 2 0 01-2 2H5a2 2 0 01-2-2v-6a2 2 0 012-2m14 0V9a2 2 0 00-2-2M5 11V9a2 2 0 012-2m0 0V5a2 2 0 012-2h6a2 2 0 012 2v2M7 7h10"
										/>
									</svg>
									<span class="font-mono text-sm text-surface-700 dark:text-surface-300"
										>{input}</span
									>
								</a>
							</li>
						{/each}
					</ul>
				{/if}
			{:catch}
				<p class="text-red-500">Failed to load inputs</p>
			{/await}
		</div>

		<div class="card">
			<h2 class="mb-4 text-lg font-semibold text-surface-900 dark:text-white">Outputs</h2>

			{#await data.outputs}
				<div class="space-y-2">
					{#each Array(3) as _}
						<div class="skeleton h-8 w-full rounded"></div>
					{/each}
				</div>
			{:then outputs}
				{#if outputs.length === 0}
					<p class="text-surface-500 dark:text-surface-400">No outputs</p>
				{:else}
					<ul class="space-y-1">
						{#each outputs as output}
							<li>
								<a
									href={`/graphs/${data.graph}/nodes/${data.node}/params/${output}`}
									class="flex items-center gap-2 rounded-lg px-3 py-2 transition-colors hover:bg-surface-100 dark:hover:bg-surface-700"
								>
									<svg
										class="h-4 w-4 text-surface-400"
										fill="none"
										viewBox="0 0 24 24"
										stroke="currentColor"
									>
										<path
											stroke-linecap="round"
											stroke-linejoin="round"
											stroke-width="2"
											d="M4 16v1a3 3 0 003 3h10a3 3 0 003-3v-1m-4-8l-4-4m0 0L8 8m4-4v12"
										/>
									</svg>
									<span class="font-mono text-sm text-surface-700 dark:text-surface-300"
										>{output}</span
									>
								</a>
							</li>
						{/each}
					</ul>
				{/if}
			{:catch}
				<p class="text-red-500">Failed to load outputs</p>
			{/await}
		</div>
	</div>
</div>
