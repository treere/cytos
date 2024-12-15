<script lang="ts">
	import '../app.css';
	let { children } = $props();
</script>

<div class="mx-auto h-full w-full max-w-screen-lg bg-white p-6 shadow-lg">
	<div>
		<a href="/">Graph</a>
		<a href="/map">Map</a>
	</div>
	<div class="mt-8 flex-grow justify-center">
		{@render children()}
	</div>
</div>
