<script lang="ts">
	import { JSONEditor, Mode, type Content } from 'svelte-jsoneditor';
	import { invalidate } from '$app/navigation';
	import ImageViewer from '$lib/image_viewer.svelte';

	interface Props {
		data: {
			value: Promise<any>;
			graph: string;
			node: string;
			param: string;
		};
	}

	let { data }: Props = $props();

	let content: Content | undefined = $state();
	let activeTab = $state<'preview' | 'json'>('json');
	let isLoading = $state(false);
	let toast = $state<{ message: string; type: 'success' | 'error' } | null>(null);
	let hasChanges = $state(false);

	function showToast(message: string, type: 'success' | 'error') {
		toast = { message, type };
		setTimeout(() => {
			toast = null;
		}, 3000);
	}

	async function modify(endpoint: string, actionName: string) {
		if (content === undefined && endpoint !== 'dump') {
			showToast('No content to modify', 'error');
			return;
		}

		isLoading = true;
		try {
			let body: string;
			if (endpoint === 'dump') {
				body = '';
			} else if (content && 'text' in content && content.text !== undefined) {
				body = content.text;
			} else if (
				content &&
				'json' in content &&
				content.json !== undefined &&
				content.json !== null
			) {
				body = JSON.stringify(content.json);
			} else {
				showToast('Invalid content', 'error');
				isLoading = false;
				return;
			}

			const response = await fetch(
				`/api/graphs/${data.graph}/nodes/${data.node}/params/${data.param}/${endpoint}`,
				{
					method: 'POST',
					body,
					headers: {
						'Content-Type': 'application/json'
					}
				}
			);

			if (response.ok) {
				showToast(`${actionName} successful`, 'success');
				if (endpoint === 'assign') {
					hasChanges = false;
				}
			} else {
				showToast(`${actionName} failed`, 'error');
			}
		} catch {
			showToast(`${actionName} failed`, 'error');
		} finally {
			isLoading = false;
		}
	}

	const assign = () => modify('assign', 'Assign');
	const load = () => modify('load', 'Load');

	const update = async () => {
		isLoading = true;
		try {
			await invalidate(`/api/graphs/${data.graph}/nodes/${data.node}/params/${data.param}/dump`);
			showToast('Reloaded', 'success');
		} catch {
			showToast('Reload failed', 'error');
		} finally {
			isLoading = false;
		}
	};

	function handleContentChange(newContent: Content) {
		content = newContent;
		hasChanges = true;
	}

	function getJsonType(value: any): string {
		if (value === null) return 'null';
		if (Array.isArray(value)) return 'array';
		return typeof value;
	}

	function getJsonSize(value: any): string {
		const str = JSON.stringify(value);
		if (str.length < 1024) return `${str.length} B`;
		if (str.length < 1024 * 1024) return `${(str.length / 1024).toFixed(1)} KB`;
		return `${(str.length / (1024 * 1024)).toFixed(1)} MB`;
	}
</script>

<div class="animate-fade-in">
	<nav class="mb-6 flex flex-wrap items-center gap-2 text-sm">
		<a href="/" class="link">root</a>
		<span class="text-surface-300 dark:text-surface-600">/</span>
		<a href={`/graphs/${data.graph}`} class="link">{data.graph}</a>
		<span class="text-surface-300 dark:text-surface-600">/</span>
		<a href={`/graphs/${data.graph}/nodes/${data.node}`} class="link">{data.node}</a>
		<span class="text-surface-300 dark:text-surface-600">/</span>
		<span class="font-mono font-medium text-surface-900 dark:text-white">{data.param}</span>
		{#if hasChanges}
			<span class="badge badge-primary">Modified</span>
		{/if}
	</nav>

	{#if toast}
		<div
			class="fixed bottom-4 right-4 z-50 animate-slide-up rounded-lg px-4 py-3 shadow-lg
				{toast.type === 'success' ? 'bg-green-500 text-white' : 'bg-red-500 text-white'}"
		>
			{toast.message}
		</div>
	{/if}

	{#await data.value}
		<div class="card">
			<div class="space-y-4">
				<div class="skeleton h-48 w-full rounded-lg"></div>
				<div class="flex gap-3">
					<div class="skeleton h-10 w-24 rounded-lg"></div>
					<div class="skeleton h-10 w-24 rounded-lg"></div>
					<div class="skeleton h-10 w-24 rounded-lg"></div>
				</div>
				<div class="skeleton h-64 w-full rounded-lg"></div>
			</div>
		</div>
	{:then value}
		<div class="grid gap-6 lg:grid-cols-4">
			<div class="space-y-6 lg:col-span-3">
				<div class="card overflow-hidden p-0">
					<div class="flex border-b border-surface-200 dark:border-surface-700">
						<button
							onclick={() => (activeTab = 'json')}
							class="flex-1 px-4 py-3 text-sm font-medium transition-colors
								{activeTab === 'json'
								? 'border-b-2 border-primary-500 text-primary-600 dark:text-primary-400'
								: 'text-surface-500 hover:text-surface-700 dark:text-surface-400 dark:hover:text-surface-200'}"
						>
							JSON Editor
						</button>
						<button
							onclick={() => (activeTab = 'preview')}
							class="flex-1 px-4 py-3 text-sm font-medium transition-colors
								{activeTab === 'preview'
								? 'border-b-2 border-primary-500 text-primary-600 dark:text-primary-400'
								: 'text-surface-500 hover:text-surface-700 dark:text-surface-400 dark:hover:text-surface-200'}"
						>
							Preview
						</button>
					</div>

					<div class="min-h-[400px]">
						{#if activeTab === 'json'}
							<JSONEditor
								mode={Mode.tree}
								content={{ json: value }}
								onChange={handleContentChange}
							/>
						{:else}
							<div class="p-4">
								<ImageViewer {value} />
							</div>
						{/if}
					</div>
				</div>
			</div>

			<div class="space-y-6">
				<div class="card">
					<h3
						class="mb-4 text-sm font-semibold uppercase tracking-wide text-surface-500 dark:text-surface-400"
					>
						Parameter Info
					</h3>
					<dl class="space-y-3 text-sm">
						<div>
							<dt class="text-surface-400 dark:text-surface-500">Name</dt>
							<dd class="font-mono font-medium text-surface-900 dark:text-white">{data.param}</dd>
						</div>
						<div>
							<dt class="text-surface-400 dark:text-surface-500">Type</dt>
							<dd>
								<span class="badge badge-primary">{getJsonType(value)}</span>
							</dd>
						</div>
						<div>
							<dt class="text-surface-400 dark:text-surface-500">Size</dt>
							<dd class="text-surface-700 dark:text-surface-300">{getJsonSize(value)}</dd>
						</div>
					</dl>
				</div>

				<div class="card">
					<h3
						class="mb-4 text-sm font-semibold uppercase tracking-wide text-surface-500 dark:text-surface-400"
					>
						Actions
					</h3>
					<div class="space-y-2">
						<button
							onclick={update}
							disabled={isLoading}
							class="btn btn-secondary w-full justify-start"
						>
							{#if isLoading}
								<svg class="h-4 w-4 animate-spin" fill="none" viewBox="0 0 24 24">
									<circle
										class="opacity-25"
										cx="12"
										cy="12"
										r="10"
										stroke="currentColor"
										stroke-width="4"
									></circle>
									<path
										class="opacity-75"
										fill="currentColor"
										d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4z"
									></path>
								</svg>
							{:else}
								<svg class="h-4 w-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
									<path
										stroke-linecap="round"
										stroke-linejoin="round"
										stroke-width="2"
										d="M4 4v5h.582m15.356 2A8.001 8.001 0 004.582 9m0 0H9m11 11v-5h-.581m0 0a8.003 8.003 0 01-15.357-2m15.357 2H15"
									/>
								</svg>
							{/if}
							Refresh
						</button>

						<button
							onclick={load}
							disabled={isLoading}
							class="btn btn-secondary w-full justify-start"
						>
							<svg class="h-4 w-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
								<path
									stroke-linecap="round"
									stroke-linejoin="round"
									stroke-width="2"
									d="M4 16v1a3 3 0 003 3h10a3 3 0 003-3v-1m-4-4l-4 4m0 0l-4-4m4 4V4"
								/>
							</svg>
							Load from Node
						</button>

						<button
							onclick={assign}
							disabled={isLoading || !hasChanges}
							class="btn btn-primary w-full justify-start"
						>
							<svg class="h-4 w-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
								<path
									stroke-linecap="round"
									stroke-linejoin="round"
									stroke-width="2"
									d="M5 13l4 4L19 7"
								/>
							</svg>
							Assign to Node
						</button>
					</div>
				</div>

				<div class="card">
					<h3
						class="mb-4 text-sm font-semibold uppercase tracking-wide text-surface-500 dark:text-surface-400"
					>
						Quick Tips
					</h3>
					<ul class="space-y-2 text-xs text-surface-500 dark:text-surface-400">
						<li class="flex items-start gap-2">
							<span class="text-primary-500">•</span>
							<span><strong>Load</strong> reads value from the node</span>
						</li>
						<li class="flex items-start gap-2">
							<span class="text-primary-500">•</span>
							<span><strong>Assign</strong> writes value to the node</span>
						</li>
						<li class="flex items-start gap-2">
							<span class="text-primary-500">•</span>
							<span><strong>Refresh</strong> reloads from server</span>
						</li>
					</ul>
				</div>
			</div>
		</div>
	{:catch}
		<div class="card border-red-200 bg-red-50 dark:border-red-800 dark:bg-red-900/20">
			<p class="text-red-600 dark:text-red-400">Failed to load parameter value</p>
		</div>
	{/await}
</div>
