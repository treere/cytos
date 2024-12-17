<script lang="ts">
	import { VisSingleContainer, VisGraph } from '@unovis/svelte';
	import type { Link, Node } from './+page';

	interface Props {
		data: {
			graph: Promise<{ nodes: Node[]; links: Link[] }>;
			nodeLabel: any;
			nodeFill: any;
			linkStroke: any;
			linkFlow: any;
		};
	}
	const { data }: Props = $props();
</script>

{#await data.graph}
	Loading
{:then graph}
	<VisSingleContainer data={graph}>
		<VisGraph
			nodeLabel={data.nodeLabel}
			nodeFill={data.nodeFill}
			linkStroke={data.linkStroke}
			linkFlow={data.linkFlow}
		/>
	</VisSingleContainer>

	<h1 class="text-2xl">Nodi</h1>
	<ul class="list-inside list-disc">
		{#each graph.nodes as node}
			<li>{node.type}: <a class="hover:underline" href={node.link} target="_blank"> {node.id} </a></li>
		{/each}
	</ul>
{:catch}
	Error
{/await}
