<script lang="ts">
	interface Props {
		data: {
			graphs: Promise<string[]>;
		};
	}

	let { data }: Props = $props();
</script>

<h1 class="mb-3 text-5xl">Graph list</h1>

{#await data.graphs}
	loading
{:then graphs}
	<ul>
		{#each graphs as graph}
			<li><a href={`graphs/${graph}`}>{graph}</a></li>
		{/each}
	</ul>
{:catch}
	Error
{/await}
