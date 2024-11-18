<script lang="ts">
	let { data } = $props();
</script>

<div>
	Graph: {data.graph}
</div>

<div>
	Node: {data.node}
</div>

<div>
	Inputs: {#await data.inputs}
		loading
	{:then inputs}
		<ul>
			{#each inputs as input}
				<li><a href={`/graphs/${data.graph}/nodes/${data.node}/params/${input}`}>{input}</a></li>
			{/each}
		</ul>
	{:catch}
		Error
	{/await}
</div>

<div>
	Outputs: {#await data.outputs}
		loading
	{:then outputs}
		<ul>
			{#each outputs as output}
				<li><a href={`/graphs/${data.graph}/nodes/${data.node}/params/${output}`}>{output}</a></li>
			{/each}
		</ul>
	{:catch}
		Error
	{/await}
</div>
