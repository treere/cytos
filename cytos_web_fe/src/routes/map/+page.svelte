<script lang="ts">
	import '@xyflow/svelte/dist/style.css';
	import {
		SvelteFlow,
		Background,
		Controls,
		MiniMap,
		type Node as FlowNode,
		type Edge as FlowEdge
	} from '@xyflow/svelte';
	import type { Link, Node } from './+page';
	import { createReceiver, deleteReceiver, linkNodes } from '$lib/api';
	import CustomNode from './CustomNode.svelte';

	const nodeTypes = { custom: CustomNode };

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
		return x == 'Graph'
			? 'text-red-500 dark:text-red-400'
			: x == 'Node'
				? 'text-green-500 dark:text-green-400'
				: 'text-blue-500 dark:text-blue-400';
	};

	let source = $state('');
	let target = $state('');

	async function getFlowData() {
		const graph = await data.graph;
		const nodeMap = new Map<string, { x: number; y: number }>();
		const levels: Map<string, number> = new Map();

		graph.nodes.forEach((n) => {
			const depth = n.id.split('/').length - 1;
			levels.set(n.id, depth);
		});

		const levelCounts: Map<number, number> = new Map();

		graph.nodes.forEach((n) => {
			const level = levels.get(n.id) ?? 0;
			const count = levelCounts.get(level) ?? 0;
			levelCounts.set(level, count + 1);
			nodeMap.set(n.id, { x: level * 200, y: count * 80 });
		});

		const nodeIds = new Set(graph.nodes.map((n) => n.id));

		const flowNodes: FlowNode[] = graph.nodes.map((n) => {
			const pos = nodeMap.get(n.id) ?? { x: 0, y: 0 };
			return {
				id: n.id,
				position: pos,
				data: { label: n.label, color: n.color, type: n.type },
				type: 'custom'
			};
		});

		const flowEdges: FlowEdge[] = graph.links
			.filter((l) => nodeIds.has(l.source) && nodeIds.has(l.target))
			.map((l, i) => ({
				id: `e${i}`,
				source: l.source,
				target: l.target,
				animated: l.active,
				style: `stroke: ${l.color}`
			}));

		return { nodes: flowNodes, edges: flowEdges };
	}
</script>

<div class="animate-fade-in">
	<div class="mb-6 flex items-center gap-2 text-sm">
		<a
			href="/"
			class="text-surface-500 hover:text-primary-600 dark:text-surface-400 dark:hover:text-primary-400"
			>root</a
		>
		<span class="text-surface-300 dark:text-surface-600">/</span>
		<span class="font-medium text-surface-900 dark:text-white">Map</span>
	</div>

	{#await data.graph}
		<div class="grid gap-6 lg:grid-cols-3">
			<div class="card lg:col-span-1">
				<div class="space-y-3">
					<div class="skeleton h-6 w-24 rounded"></div>
					{#each Array(5) as _}
						<div class="skeleton h-8 w-full rounded"></div>
					{/each}
				</div>
			</div>
			<div class="card lg:col-span-2">
				<div class="skeleton h-96 w-full rounded-lg"></div>
			</div>
		</div>
	{:then graph}
		{@const flowData = getFlowData()}
		<div class="grid gap-6 lg:grid-cols-3">
			<div class="card lg:col-span-1">
				<h2 class="mb-4 text-lg font-semibold text-surface-900 dark:text-white">Nodes</h2>
				<div class="max-h-96 space-y-1 overflow-y-auto">
					{#each graph.nodes as node}
						<a
							href={node.link}
							target="_blank"
							class="flex items-center gap-2 rounded-lg px-2 py-1.5 transition-colors hover:bg-surface-100 dark:hover:bg-surface-700"
						>
							<span class={typeToColor(node.type)}>{node.type}</span>
							<span class="font-mono text-sm text-surface-700 dark:text-surface-300">{node.id}</span
							>
						</a>
					{/each}
				</div>
			</div>

			<div class="card lg:col-span-2">
				<div class="h-96">
					{#await flowData}
						<div class="skeleton h-full w-full rounded-lg"></div>
					{:then flow}
						<SvelteFlow nodes={flow.nodes} edges={flow.edges} {nodeTypes} fitView>
							<Background />
							<Controls />
							<MiniMap />
						</SvelteFlow>
					{/await}
				</div>
			</div>
		</div>

		<div class="card mt-6">
			<h2 class="mb-4 text-lg font-semibold text-surface-900 dark:text-white">Actions</h2>
			<form class="flex flex-wrap gap-4" onsubmit={(e) => e.preventDefault()}>
				<div class="min-w-[150px] flex-1">
					<label
						for="source"
						class="mb-1 block text-sm font-medium text-surface-700 dark:text-surface-300"
						>Source</label
					>
					<input
						id="source"
						type="text"
						placeholder="source node"
						bind:value={source}
						class="input"
					/>
				</div>
				<div class="min-w-[150px] flex-1">
					<label
						for="target"
						class="mb-1 block text-sm font-medium text-surface-700 dark:text-surface-300"
						>Target</label
					>
					<input
						id="target"
						type="text"
						placeholder="target node"
						bind:value={target}
						class="input"
					/>
				</div>
				<div class="flex w-full items-end gap-3">
					<button onclick={() => linkNodes(source, target)} class="btn btn-primary"> Link </button>
					<button onclick={() => createReceiver(source, target)} class="btn btn-secondary">
						Create Receiver
					</button>
					<button onclick={() => deleteReceiver(source, target)} class="btn btn-danger">
						Delete Receiver
					</button>
				</div>
			</form>
		</div>
	{:catch}
		<div class="card border-red-200 bg-red-50 dark:border-red-800 dark:bg-red-900/20">
			<p class="text-red-600 dark:text-red-400">Failed to load graph data</p>
		</div>
	{/await}
</div>
