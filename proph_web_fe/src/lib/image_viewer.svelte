<script lang="ts">
	let { value }: { value: any } = $props();

	let name = $state('');
</script>

<input type="text" bind:value={name} />
<img id="image" alt="frame" />
<button
	onclick={() => {
		console.info(name, value[name]);
		const binString = Array.from(value[name], (byte: number) => String.fromCodePoint(byte)).join(
			''
		);
		const b64encoded = btoa(binString);

		document.getElementById('image')?.setAttribute('src', 'data:image/jpg;base64,' + b64encoded);
	}}>view</button
>
