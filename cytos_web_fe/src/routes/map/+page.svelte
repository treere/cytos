<script lang="ts">
	import { VisSingleContainer, VisGraph } from '@unovis/svelte';

	interface Props {
		data: any;
		nodeLabel: any;
		nodeFill: any;
		linkStroke: any;
		linkFlow: any;
	}
	const { data }: Props = $props();
</script>

<VisSingleContainer data={data.data}>
	<VisGraph
		nodeLabel={data.nodeLabel}
		nodeFill={data.nodeFill}
		linkStroke={data.linkStroke}
		linkFlow={data.linkFlow}
	/>
</VisSingleContainer>
