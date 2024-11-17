<script>
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
				<li>{input}</li>
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
				<li>{output}</li>
			{/each}
		</ul>
	{:catch}
		Error
	{/await}
</div>
