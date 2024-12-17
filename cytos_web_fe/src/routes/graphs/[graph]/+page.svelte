<script lang="ts">
	import { graphStart, graphStop } from '$lib/api.js';

	let { data } = $props();
</script>

<h1 class="mb-4 text-3xl">
	>
	<a href="/">root</a> >
	<a href={`/graphs/${data.graph}`}>{data.graph}</a>
</h1>

<div class="mb-4">
	<h2 class="mb-4 text-2xl">Stato</h2>

	{#await data.status}
		loading
	{:then status}
		<h3 class="mb-4 text-xl font-bold">{status}</h3>
		<div class="mt-2">
			<button
				onclick={() => graphStart(data.graph)}
				class="mr-2 rounded-lg bg-blue-500 px-4 py-2 font-bold text-white hover:bg-blue-600"
			>
				Start
			</button>
			<button
				onclick={() => graphStop(data.graph)}
				class="rounded-lg bg-red-500 px-4 py-2 font-bold text-white hover:bg-red-600"
			>
				Stop
			</button>
		</div>
	{:catch}
		Error
	{/await}
</div>

<h2 class="mb-4 text-2xl">Nodi</h2>
<ul class="list-inside list-disc pl-4">
	{#await data.nodes}
		loading
	{:then nodes}
		{#each nodes as node}
			<li><a href={`/graphs/${data.graph}/nodes/${node}`}>{node}</a></li>
		{/each}
	{:catch}
		Error
	{/await}
</ul>
