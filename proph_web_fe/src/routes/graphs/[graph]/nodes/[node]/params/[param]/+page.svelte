<script lang="ts">
	import { JSONEditor, Mode, type Content } from 'svelte-jsoneditor';
	import { invalidateAll } from '$app/navigation';
	let { data } = $props();

	let content: Content | undefined = $state();

	const load = () => {
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

	const update = () => {
		invalidateAll();
	};
</script>

<h1 class="mb-4 text-3xl">
  >
	<a href="/">root</a> >
	<a href={`/graphs/${data.graph}`}>{data.graph}</a> >
	<a href={`/graphs/${data.graph}/nodes/${data.node}`}>{data.node}</a> >
    <a href={`/graphs/${data.graph}/nodes/${data.node}/params/${data.param}`}>{data.param}</a> 
</h1>

<div class="mb-4">
	{#await data.value}
		loading
	{:then value}
		<JSONEditor
			mode={Mode.text}
			content={{ json: value }}
			onChange={(data) => {
				content = data;
			}}
		/>
		<button
			onclick={update}
			class="mr-2 mt-3 rounded-lg bg-blue-500 px-4 py-2 font-bold text-white hover:bg-blue-600"
		>
			Update
		</button>
		<button
			onclick={load}
			class="mr-2 mt-3 rounded-lg bg-blue-500 px-4 py-2 font-bold text-white hover:bg-blue-600"
		>
			Load
		</button>
	{:catch}
		Error
	{/await}
</div>
