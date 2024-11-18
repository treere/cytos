import type { PageLoad } from './$types';

export const load: PageLoad = async ({ fetch, params }) => {
	const graph = params.graph;
	const node = params.node;
	return {
		graph,
		node,
		inputs: fetch(`/api/graphs/${graph}/nodes/${node}/inputs`).then((x) => x.json()),
		outputs: fetch(`/api/graphs/${graph}/nodes/${node}/outputs`).then((x) => x.json())
	};
};
