import type { PageLoad } from './$types';

export const load: PageLoad = async ({ fetch, params }) => {
	const graph = params.graph;
	const node = params.node;
	const description = fetch(`/api/graphs/${graph}/nodes/${node}/describe`).then((x) => x.json());
	const parameters = description.then((x) => x.params);
	const inputs = parameters.then((x) =>
		x.filter((y) => y.directions.includes('Input')).map((x) => x.description)
	);
	const outputs = parameters.then((x) =>
		x.filter((y) => y.directions.includes('Output')).map((x) => x.description)
	);
	return {
		graph,
		node,
		inputs,
		outputs
	};
};
