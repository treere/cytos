import type { PageLoad } from './$types';

export const load: PageLoad = async ({ fetch }) => {
	const compose = async () => {
		const graphNames: string[] = await fetch('/api/graphs').then((x) => x.json());

		let nodes = await Promise.all(
			graphNames.map(async (graph) => {
				let x = await fetch(`/api/graphs/${graph}/nodes`);
				return [graph, await x.json()] as const;
			})
		);

		const panels = nodes.map(([g, n]) => ({ nodes: n, label: g }));

		return {
			panels,
			data: { nodes: nodes.flatMap(([_, n]) => n).map((n) => ({ id: n })), links: [] }
		};
	};

	const res = await compose();
	console.info(res);
	return res;
};
