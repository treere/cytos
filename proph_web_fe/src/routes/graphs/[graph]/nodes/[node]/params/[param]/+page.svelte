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
	Param: {data.param}
</div>

<div>
	Value: {#await data.value}
		loading
	{:then value}
		{JSON.stringify(value[0], null, 2)}
	{:catch}
		Error
	{/await}
</div>
