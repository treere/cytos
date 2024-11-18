<script lang="ts">
	import { JSONEditor, Mode, type Content, type TextContent } from 'svelte-jsoneditor';
	let { data } = $props();

	let content: Content | undefined = $state();

	const update = () => {
		if (content !== undefined && 'text' in content && content.text !== undefined) {
			fetch(`/api/graphs/${data.graph}/nodes/${data.node}/params/${data.param}/load`, {
				method: 'POST',
				body: content.text,
				headers: {
					'Content-Type': 'application/json'
				}
			});
		} else {
			console.log('Invalid content');
		}
	};
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
		<JSONEditor
			mode={Mode.text}
			content={{ json: value }}
			onChange={(data) => {
				content = data;
			}}
		/>
		<button onclick={update}>update</button>
	{:catch}
		Error
	{/await}
</div>
