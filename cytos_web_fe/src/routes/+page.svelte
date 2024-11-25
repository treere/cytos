<script lang="ts">
	interface Props {
		data: {
			graphs: Promise<string[]>;
		};
	}

	let { data }: Props = $props();
</script>

<h1 class="mb-4 text-3xl">
	>
	<a href="/">root</a>
</h1>
<div class="mb-4">
	<h2 class="mb-4 text-2xl">Graphs</h2>

	{#await data.graphs}
		Loading
	{:then graphs}
		<ul class="list-inside list-disc pl-4">
			{#each graphs as graph}
				<li><a href={`graphs/${graph}`}>{graph}</a></li>
			{/each}
		</ul>
	{:catch}
		Error
	{/await}
</div>
