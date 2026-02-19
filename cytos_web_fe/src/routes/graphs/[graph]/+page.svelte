<script lang="ts">
	import { graphStart, graphStop } from '$lib/api.js';

	let { data } = $props();
</script>

<div class="animate-fade-in">
	<nav class="mb-6 flex items-center gap-2 text-sm">
		<a
			href="/"
			class="text-surface-500 hover:text-primary-600 dark:text-surface-400 dark:hover:text-primary-400"
			>root</a
		>
		<span class="text-surface-300 dark:text-surface-600">/</span>
		<span class="font-medium text-surface-900 dark:text-white">{data.graph}</span>
	</nav>

	<div class="grid gap-6">
		<div class="card">
			<h2 class="mb-4 text-lg font-semibold text-surface-900 dark:text-white">Status</h2>

			{#await data.status}
				<div class="skeleton h-8 w-32 rounded"></div>
			{:then status}
				<div class="mb-4 flex items-center gap-3">
					<span
						class="h-3 w-3 rounded-full {status === 'Running'
							? 'animate-pulse bg-green-500'
							: 'bg-surface-400'}"
					></span>
					<span class="font-medium text-surface-700 dark:text-surface-300">{status}</span>
				</div>
				<div class="flex gap-3">
					<button onclick={() => graphStart(data.graph)} class="btn btn-primary">
						<svg class="h-4 w-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
							<path
								stroke-linecap="round"
								stroke-linejoin="round"
								stroke-width="2"
								d="M14.752 11.168l-3.197-2.132A1 1 0 0010 9.87v4.263a1 1 0 001.555.832l3.197-2.132a1 1 0 000-1.664z"
							/>
							<path
								stroke-linecap="round"
								stroke-linejoin="round"
								stroke-width="2"
								d="M21 12a9 9 0 11-18 0 9 9 0 0118 0z"
							/>
						</svg>
						Start
					</button>
					<button onclick={() => graphStop(data.graph)} class="btn btn-danger">
						<svg class="h-4 w-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
							<path
								stroke-linecap="round"
								stroke-linejoin="round"
								stroke-width="2"
								d="M21 12a9 9 0 11-18 0 9 9 0 0118 0z"
							/>
							<path
								stroke-linecap="round"
								stroke-linejoin="round"
								stroke-width="2"
								d="M9 10a1 1 0 011-1h4a1 1 0 011 1v4a1 1 0 01-1 1h-4a1 1 0 01-1-1v-4z"
							/>
						</svg>
						Stop
					</button>
				</div>
			{:catch}
				<p class="text-red-500">Failed to load status</p>
			{/await}
		</div>

		<div class="card">
			<h2 class="mb-4 text-lg font-semibold text-surface-900 dark:text-white">Nodes</h2>

			{#await data.nodes}
				<div class="space-y-2">
					{#each Array(5) as _}
						<div class="skeleton h-10 w-full rounded"></div>
					{/each}
				</div>
			{:then nodes}
				{#if nodes.length === 0}
					<p class="text-surface-500 dark:text-surface-400">No nodes found</p>
				{:else}
					<ul class="divide-y divide-surface-200 dark:divide-surface-700">
						{#each nodes as node}
							<li>
								<a
									href={`/graphs/${data.graph}/nodes/${node}`}
									class="flex items-center justify-between py-3 transition-colors hover:text-primary-600 dark:hover:text-primary-400"
								>
									<span class="font-mono text-sm text-surface-700 dark:text-surface-300"
										>{node}</span
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
											d="M9 5l7 7-7 7"
										/>
									</svg>
								</a>
							</li>
						{/each}
					</ul>
				{/if}
			{:catch}
				<p class="text-red-500">Failed to load nodes</p>
			{/await}
		</div>
	</div>
</div>
