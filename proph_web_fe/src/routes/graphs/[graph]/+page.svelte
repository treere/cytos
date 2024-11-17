<script lang="ts">
	let { data } = $props();

	async function handleStart(name: string) {
		await fetch(`/api/graphs/${name}/start`, { method: 'POST' });
	}
	async function handleStop(name: string) {
		await fetch(`/api/graphs/${name}/stop`, { method: 'POST' });
	}
</script>

<div>
	Nome: {data.name}
</div>
<div>
	Stato: {#await data.status}
		loading
	{:then status}
		{status}
		<button onclick={() => handleStart(data.name)}>start</button>
		<button onclick={() => handleStop(data.name)}>stop</button>
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
				<li><a href={`/graphs/${data.name}/nodes/${node}`}>{node}</a></li>
			{/each}
		</ul>
	{:catch}
		Error
	{/await}
</div>
