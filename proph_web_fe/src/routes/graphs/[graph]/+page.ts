import type { PageLoad } from './$types';

export const load: PageLoad = async ({ fetch, params }) => {
	const name = params.graph;
	return {
		name,
		status: fetch(`/api/graphs/${name}`).then((x) => x.json()),
		nodes: fetch(`/api/graphs/${name}/nodes`).then((x) => x.json())
	};
};
