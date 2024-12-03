<script lang="ts">
	import { VisSingleContainer, VisGraph } from '@unovis/svelte';
	import type { GraphInputNode } from '@unovis/ts';

	const panels = [
		{ nodes: ['1', '2'], label: 'A' },
		{ nodes: ['3', '4'], label: 'B' }
	];
	const nodes = [{ id: '1' }, { id: '2' }, { id: '3' }, { id: '4' }];
	const links = [
		{ source: '1', target: '2' },
		{ source: '2', target: '3' }
	];
	const nodeLabel = (n: GraphInputNode) => n.id;

	const data = { nodes, links };
</script>

<VisSingleContainer {data}>
	<VisGraph layoutType="horizontal" {panels} {nodeLabel} />
</VisSingleContainer>
