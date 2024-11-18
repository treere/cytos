<script>
	import { JSONEditor } from 'svelte-jsoneditor';
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
		<JSONEditor content={{ json: value }} />
	{:catch}
		Error
	{/await}
</div>
