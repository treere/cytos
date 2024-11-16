<script lang="ts">
	import '../app.css';
	let { children } = $props();
</script>

<div class="mx-auto max-w-screen-lg">
	{@render children()}
</div>
