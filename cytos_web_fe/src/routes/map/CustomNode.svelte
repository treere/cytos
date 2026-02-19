<script lang="ts">
	import { Handle } from '@xyflow/svelte';

	interface Props {
		data: {
			label: string;
			color: string;
			type: string;
		};
		id: string;
	}

	let { data, id }: Props = $props();

	const isGraph = data.type === 'Graph';
	const isNode = data.type === 'Node';
	const isInput = data.type === 'Input';
	const isOutput = data.type === 'Output';
</script>

{#if isGraph}
	<div
		class="rounded-xl border-2 border-red-400 bg-red-50 px-4 py-3 text-sm font-bold text-red-700 shadow-lg dark:border-red-600 dark:bg-red-900/30 dark:text-red-300"
	>
		<Handle type="source" position="right" id="out" />
		📁 {data.label}
	</div>
{:else if isNode}
	<div
		class="rounded-lg border border-green-400 bg-green-50 px-3 py-2 text-sm font-semibold text-green-700 shadow-md dark:border-green-600 dark:bg-green-900/30 dark:text-green-300"
	>
		<Handle type="target" position="left" id="in" class="!bg-green-500" />
		<Handle type="source" position="right" id="out" class="!bg-green-500" />
		🔧 {data.label}
	</div>
{:else if isInput}
	<div
		class="min-w-[80px] rounded-full border-2 border-blue-400 bg-blue-50 px-3 py-2 text-center text-sm font-medium text-blue-700 shadow-sm dark:border-blue-500 dark:bg-blue-900/30 dark:text-blue-300"
	>
		<Handle type="target" position="left" id="in" class="!h-3 !w-3 !bg-blue-500" />
		↓ {data.label}
	</div>
{:else if isOutput}
	<div
		class="min-w-[80px] rounded-full border-2 border-orange-400 bg-orange-50 px-3 py-2 text-center text-sm font-medium text-orange-700 shadow-sm dark:border-orange-500 dark:bg-orange-900/30 dark:text-orange-300"
	>
		↑ {data.label}
		<Handle type="source" position="right" id="out" class="!h-3 !w-3 !bg-orange-500" />
	</div>
{:else if isOutput}
	<div
		class="rounded-full border border-orange-400 bg-orange-50 px-2 py-1 text-xs font-medium text-orange-700 shadow-sm dark:border-orange-500 dark:bg-orange-900/30 dark:text-orange-300"
	>
		↑ {data.label}
		<Handle type="source" position="right" id="out" class="!h-2 !w-2 !bg-orange-500" />
	</div>
{:else if isOutput}
	<div
		class="rounded-full border border-orange-400 bg-orange-50 px-2 py-1 text-xs font-medium text-orange-700 shadow-sm dark:border-orange-500 dark:bg-orange-900/30 dark:text-orange-300"
	>
		<Handle type="target" position="left" id="in" class="!h-2 !w-2 !bg-orange-500" />
		↑ {data.label}
	</div>
{:else if isOutput}
	<div
		class="rounded-full border border-orange-400 bg-orange-50 px-2 py-1 text-xs font-medium text-orange-700 shadow-sm dark:border-orange-500 dark:bg-orange-900/30 dark:text-orange-300"
	>
		↑ {data.label}
		<Handle type="source" position="right" id="out" class="!h-2 !w-2 !bg-orange-500" />
	</div>
{/if}
