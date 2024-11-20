<script lang="ts">
	import { rawToImageData, setChild } from '$lib';
	let { value }: { value: any } = $props();

	let selectedFormat = $state('jpeg');
	let formats = ['jpeg', 'raw grey'];

	let name = $state('');
	let width = $state('width');
	let height = $state('height');

	let renderImage = () => {
		const binString = Array.from(value[name], (byte: number) => String.fromCodePoint(byte)).join(
			''
		);
		const b64encoded = btoa(binString);
		const img = document.createElement('img');
		img.src = 'data:image/jpg;base64,' + b64encoded;
		img.style.width = '800px';

		setChild('image', img);
	};

	let renderGreyArray = () => {
		const imageData = rawToImageData(value[width], value[height], value[name]);
		console.info('B');
		const canvas = document.createElement('canvas');
		canvas.width = value[width];
		canvas.height = value[height];
		const ctx = canvas.getContext('2d');
		canvas.style.width = '800px';

		if (!ctx) return;

		ctx.putImageData(imageData, 0, 0);
		setChild('image', canvas);
	};

	let render = () => {
		if (selectedFormat === 'jpeg') renderImage();
		else renderGreyArray();
	};
</script>

<div class="mb-2">
	<select bind:value={selectedFormat}>
		{#each formats as format}
			<option value={format}>{format}</option>
		{/each}
	</select>
	<button
		class="mr-2 rounded-lg bg-blue-500 px-4 py-2 font-bold text-white hover:bg-blue-600"
		onclick={render}>View</button
	>
</div>
<div class="my-3">
	{#if selectedFormat === 'jpeg'}
		<label for="data">Data:</label>
		<input type="text" bind:value={name} id="data" />
	{:else}
		<label for="data">Data:</label>
		<input type="text" bind:value={name} id="data" />
		<label for="width">Width:</label>
		<input type="text" bind:value={width} id="width" />
		<label for="height">Height:</label>
		<input type="text" bind:value={height} id="height" />
	{/if}
</div>

<div class="my-2 max-w-screen-sm" id="image"></div>
