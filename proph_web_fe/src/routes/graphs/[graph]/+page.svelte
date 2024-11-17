<script>
	let { data } = $props();
</script>

<div>
	Nome: {data.name}
</div>
<div>
	Stato: {#await data.status}
		loading
	{:then status}
		{status}
	{:catch}
		Error
	{/await}
</div>
<div>
	Nodi: {#await data.nodes}
		loading
	{:then nodes}
		<ul>
			{#each nodes as node}
				<li>{node}</li>
			{/each}
		</ul>
	{:catch}
		Error
	{/await}
</div>
