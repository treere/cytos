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
{:catch}
	Error
{/await}
