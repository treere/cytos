<script lang="ts">
	import { JSONEditor, Mode, type Content } from 'svelte-jsoneditor';
	import { invalidate } from '$app/navigation';
	import ImageViewer from '$lib/image_viewer.svelte';
	let { data } = $props();

	let content: Content | undefined = $state();

	const modify = (endpoint: string) => {
		if (content === undefined) {
			console.log('Invalid content', content);
			return;
		}
		if ('text' in content && content.text !== undefined) {
			fetch(`/api/graphs/${data.graph}/nodes/${data.node}/params/${data.param}/${endpoint}`, {
				method: 'POST',
				body: content.text,
				headers: {
					'Content-Type': 'application/json'
				}
			});
			return;
		}

		if ('json' in content && content.json !== undefined && content.json !== null) {
			fetch(`/api/graphs/${data.graph}/nodes/${data.node}/params/${data.param}/${endpoint}`, {
				method: 'POST',
				body: JSON.stringify(content.json),
				headers: {
					'Content-Type': 'application/json'
				}
			});
			return;
		}
		console.log('Invalid content', content);
	};

	const assign = () => modify('assign');
	const load = () => modify('load');

	const update = () => {
		invalidate(`/api/graphs/${data.graph}/nodes/${data.node}/params/${data.param}/dump`);
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
		<ImageViewer {value} />
		<button
			onclick={update}
			class="my-3 mr-2 rounded-lg bg-blue-500 px-4 py-2 font-bold text-white hover:bg-blue-600"
		>
			Update
		</button>
		<button
			onclick={assign}
			class="my-3 mr-2 rounded-lg bg-blue-500 px-4 py-2 font-bold text-white hover:bg-blue-600"
		>
			Assign
		</button>
		<button
			onclick={load}
			class="my-3 mr-2 rounded-lg bg-blue-500 px-4 py-2 font-bold text-white hover:bg-blue-600"
		>
			Load
		</button>

		<JSONEditor
			mode={Mode.tree}
			content={{ json: value }}
			onChange={(data) => {
				content = data;
			}}
		/>
	{:catch}
		Error
	{/await}
</div>
