<script lang="ts">
	import { id2number, id2string } from '$lib/api';

	let value = $state('');
	let result = $state('');
	let convertToNumber = $state(false);
	let isLoading = $state(false);

	const convert = async () => {
		isLoading = true;
		try {
			result = await (convertToNumber ? id2number(value) : id2string(value));
		} catch {
			result = '<<error>>';
		} finally {
			isLoading = false;
		}
	};
</script>

<div class="animate-fade-in">
	<div class="mb-6 flex items-center gap-2 text-sm">
		<a
			href="/"
			class="text-surface-500 hover:text-primary-600 dark:text-surface-400 dark:hover:text-primary-400"
			>root</a
		>
		<span class="text-surface-300 dark:text-surface-600">/</span>
		<span class="font-medium text-surface-900 dark:text-white">Utils</span>
	</div>

	<div class="card max-w-lg">
		<h2 class="mb-4 text-lg font-semibold text-surface-900 dark:text-white">ID Converter</h2>

		<div class="space-y-4">
			<div>
				<label
					for="value"
					class="mb-1 block text-sm font-medium text-surface-700 dark:text-surface-300"
				>
					Value
				</label>
				<input id="value" type="text" placeholder="Enter ID value" bind:value class="input" />
			</div>

			<div class="flex gap-4">
				<label class="flex cursor-pointer items-center gap-2">
					<input
						type="radio"
						value={false}
						bind:group={convertToNumber}
						class="h-4 w-4 text-primary-600 focus:ring-primary-500"
					/>
					<span class="text-surface-700 dark:text-surface-300">To String</span>
				</label>

				<label class="flex cursor-pointer items-center gap-2">
					<input
						type="radio"
						value={true}
						bind:group={convertToNumber}
						class="h-4 w-4 text-primary-600 focus:ring-primary-500"
					/>
					<span class="text-surface-700 dark:text-surface-300">To Number</span>
				</label>
			</div>

			<button onclick={convert} disabled={isLoading || !value} class="btn btn-primary w-full">
				{#if isLoading}
					<svg class="h-4 w-4 animate-spin" fill="none" viewBox="0 0 24 24">
						<circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4"
						></circle>
						<path
							class="opacity-75"
							fill="currentColor"
							d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"
						></path>
					</svg>
				{/if}
				Convert
			</button>

			{#if result}
				<div class="rounded-lg bg-surface-100 p-4 dark:bg-surface-700">
					<p class="mb-1 text-xs font-medium uppercase text-surface-500 dark:text-surface-400">
						Result
					</p>
					<p class="font-mono text-surface-900 dark:text-white">{result}</p>
				</div>
			{/if}
		</div>
	</div>
</div>
