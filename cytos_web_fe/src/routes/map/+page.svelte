<script lang="ts">
	import { VisSingleContainer, VisGraph } from '@unovis/svelte';
	import type { GraphInputNode } from '@unovis/ts';

	const nodeLabel = (n: GraphInputNode) => n.id;

	interface Props {
		data: any;
		panels: any;
	}
	const { data }: Props = $props();
</script>

<VisSingleContainer data={data.data}>
	<VisGraph layoutType="horizontal" panels={data.panels} {nodeLabel} />
</VisSingleContainer>
