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
