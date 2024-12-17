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
	<div class="flex">
		<div>
			<h1 class="text-2xl">Nodi</h1>

			{#each graph.nodes as node}
				<div class="text-nowrap">
					{node.type}: <a class="hover:underline" href={node.link} target="_blank"> {node.id} </a>
				</div>
			{/each}
		</div>

		<VisSingleContainer data={graph}>
			<VisGraph
				nodeLabel={data.nodeLabel}
				nodeFill={data.nodeFill}
				linkStroke={data.linkStroke}
				linkFlow={data.linkFlow}
			/>
		</VisSingleContainer>
	</div>
{:catch}
	Error
{/await}
