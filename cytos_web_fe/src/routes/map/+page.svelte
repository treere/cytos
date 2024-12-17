<script lang="ts">
	import { VisSingleContainer, VisGraph } from '@unovis/svelte';
	import type { Link, Node } from './+page';
	import { invalidateAll } from '$app/navigation';

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
	const link = async () => {
		const s = source.split('/').filter((x) => x);
		const t = target.split('/').filter((x) => x);
		console.info('>', s, t);
		if (s.length != 3 || t.length != 3) return;
		console.info('here');
		const [s_g, s_n, s_p] = s;
		const [t_g, t_n, t_p] = t;

		if (s_g != t_g) return;
		await fetch(`/api/graphs/${s_g}/nodes/link`, {
			method: 'POST',
			body: JSON.stringify([
				[s_n, s_p],
				[t_n, t_p]
			]),
			headers: {
				'Content-Type': 'application/json'
			}
		});

		invalidateAll();
	};
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
			onclick={link}
			type="submit"
			class="mr-2 rounded-lg bg-blue-500 px-4 py-2 font-bold text-white hover:bg-blue-600"
			>Link</button
		>
	</form>
{:catch}
	Error
{/await}
