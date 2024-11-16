export async function load({ fetch }) {
	return {
		graphs: fetch('/api/graphs').then((x) => x.json())
	};
}
