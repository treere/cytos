<script lang="ts">
	import { id2number, id2string } from '$lib/api';

	let value = $state('');
	let result = $state('');
	let convertToNumber = $state(false);

	const convert = async () => {
		try {
			result = await (convertToNumber ? id2number(value) : id2string(value));
		} catch {
			result = '<<error>>';
		}
	};
</script>

<input type="text" placeholder="value" bind:value />
<label>
	<input type="radio" value={false} bind:group={convertToNumber} /> To String
</label>

<label>
	<input type="radio" value={true} bind:group={convertToNumber} /> To Number
</label>

<button
	onclick={convert}
	class="mr-2 rounded-lg bg-blue-500 px-4 py-2 font-bold text-white hover:bg-blue-600"
	>Convert</button
>

{result}
