import type { PageLoad } from './$types';

export const load: PageLoad = async ({ fetch, params }) => {
	const graph = params.graph;
	return {
		graph,
		status: fetch(`/api/graphs/${graph}`).then((x) => x.json()),
		nodes: fetch(`/api/graphs/${graph}/nodes`).then((x) => x.json())
	};
};
