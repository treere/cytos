export async function load({ fetch, params }) {
	const graph = params.graph;
	const node = params.node;
	const param = params.param;
	return {
		graph,
		node,
		param,
		value: fetch(`/api/graphs/${graph}/nodes/${node}/params/${param}/dump`)
			.then((x) => x.json())
			.then((x) => x[0])
	};
}
