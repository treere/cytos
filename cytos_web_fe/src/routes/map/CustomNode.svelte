<script lang="ts">
	import { Handle } from '@xyflow/svelte';

	interface Props {
		data: {
			label: string;
			color: string;
		};
	}

	let { data }: Props = $props();
</script>

<div
	class="rounded-lg px-3 py-2 text-sm font-medium text-white shadow-md"
	style={`background-color: ${data.color}`}
>
	<Handle type="target" position="left" />
	{data.label}
	<Handle type="source" position="right" />
</div>
