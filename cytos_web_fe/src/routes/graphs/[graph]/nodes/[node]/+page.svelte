<script lang="ts">
	let { data } = $props();
</script>

<h1 class="mb-4 text-3xl">
	>
	<a href="/">root</a> >
	<a href={`/graphs/${data.graph}`}>{data.graph}</a> >
	<a href={`/graphs/${data.graph}/nodes/${data.node}`}>{data.node}</a>
</h1>

<div class="mb-4">
	<h2 class="mb-4 text-2xl">Inputs</h2>
	{#await data.inputs}
		loading
	{:then inputs}
		<ul class="list-inside list-disc pl-4">
			{#each inputs as input}
				<li><a href={`/graphs/${data.graph}/nodes/${data.node}/params/${input}`}>{input}</a></li>
			{/each}
		</ul>
	{:catch}
		Error
	{/await}
</div>

<div class="mb-4">
	<h2 class="mb-4 text-2xl">Outputs</h2>
	{#await data.outputs}
		loading
	{:then outputs}
		<ul class="list-inside list-disc pl-4">
			{#each outputs as output}
				<li><a href={`/graphs/${data.graph}/nodes/${data.node}/params/${output}`}>{output}</a></li>
			{/each}
		</ul>
	{:catch}
		Error
	{/await}
</div>
