<script lang="ts">
	import '../app.css';
	let { children } = $props();
</script>

<div class="mx-auto h-full w-full max-w-screen-lg bg-white p-6 shadow-lg">
	<nav class="flex h-12 items-center justify-between">
		<a href="/" class="text-blue-500 hover:text-blue-700"> Home </a>
		<button
			onclick={() => window.history.back()}
			class="rounded-md bg-gray-300 px-4 py-2 text-sm font-medium hover:bg-gray-400">Back</button
		>
	</nav>

	<div class="mt-8 flex-grow justify-center">
		{@render children()}
	</div>
</div>
