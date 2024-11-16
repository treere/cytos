<script>
	let { data } = $props();
</script>

<h1 class="mb-3 text-5xl">Graph list</h1>

{#await data.graphs}
	loading
{:then graphs}
	<ul>
		{#each graphs as graph}
			<li>{graph}</li>
		{/each}
	</ul>
{:catch}
	Error
{/await}
