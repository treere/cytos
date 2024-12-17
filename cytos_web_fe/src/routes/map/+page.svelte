<script lang="ts">
	import { VisSingleContainer, VisGraph } from '@unovis/svelte';
	import type { Link, Node } from './+page';
	import { linkNodes } from '$lib/api';

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

	const typeToColor = (x: Node['type']) => {
		return x == 'Graph' ? 'text-red-500' : x == 'Node' ? 'text-green-500' : 'text-blue-500';
	};

	let source = $state('');
	let target = $state('');
</script>

{#await data.graph}
	Loading
{:then graph}
	<div class="mb-5 flex">
		<div>
			<h1 class="text-2xl">Nodi</h1>

			{#each graph.nodes as node}
				<div class="text-nowrap">
					<span class={typeToColor(node.type)}>{node.type}</span>:
					<a class="hover:underline" href={node.link} target="_blank"> {node.id} </a>
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

	<form>
		<input type="text" placeholder="source" bind:value={source} />
		<input type="text" placeholder="target" bind:value={target} />
		<button
			onclick={() => linkNodes(source, target)}
			type="submit"
			class="mr-2 rounded-lg bg-blue-500 px-4 py-2 font-bold text-white hover:bg-blue-600"
			>Link</button
		>
	</form>
{:catch}
	Error
{/await}
