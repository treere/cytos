import type { PageLoad } from './$types';

export const load: PageLoad = async ({ fetch }) => {
	const compose = async () => {
		interface Link {
			source: string;
			target: string;
			color?: string;
		}

		interface Node {
			id: string;
		}
		const data = await fetch('/api/graphs');
		const graphs: Node[] = (await data.json()).map((x: string) => ({ id: x }));

		const receivers: Link[] = (
			await Promise.all(
				graphs.map(async (graph) => {
					const data = await fetch(`/api/graphs/${graph.id}/receivers`);
					const json: string[][][] = await data.json();

					return json.map(([src, dst]) => {
						return [src.join('/'), `${graph.id}/${dst.join('/')}`];
					});
				})
			)
		)
			.flatMap((x) => x)
			.map(([a, b]) => ({ source: a, target: b, color: '#0000ff' }));

		const processors = (
			await Promise.all(
				graphs.map(async (graph) => {
					const data = await fetch(`/api/graphs/${graph.id}/nodes`);
					const json = (await data.json()) as string[];
					return json.map((node) => ({ id: `${graph.id}/${node}` }));
				})
			)
		).flatMap((x) => x);

		const links: Link[] = (
			await Promise.all(
				graphs.map(async (graph) => {
					const data = await fetch(`/api/graphs/${graph.id}/links`);
					const json: string[][][] = await data.json();

					return json.map((x) => x.map((y) => `${graph.id}/${y.join('/')}`));
				})
			)
		)
			.flatMap((x) => x)
			.filter((x) => x.length > 1)
			.map(([a, b]) => ({ source: a, target: b, color: '#00ff00' }));

		const inputs = (
			await Promise.all(
				processors.map(async (n) => {
					const [graph, node] = n.id.split('/');
					const data = await fetch(`/api/graphs/${graph}/nodes/${node}/inputs`);
					const json = (await data.json()) as string[];
					return json.map((param) => ({ id: `${graph}/${node}/${param}` }));
				})
			)
		).flatMap((x) => x);

		const outputs = (
			await Promise.all(
				processors.map(async (n) => {
					const [graph, node] = n.id.split('/');
					const data = await fetch(`/api/graphs/${graph}/nodes/${node}/outputs`);
					const json = (await data.json()) as string[];
					return json.map((param) => ({ id: `${graph}/${node}/${param}` }));
				})
			)
		).flatMap((x) => x);

		const n = graphs.concat(graphs, inputs, outputs, processors);

		const composition: Link[] = n
			.filter((x) => x.id.includes('/'))
			.map((x) => {
				const lastIndex = x.id.lastIndexOf('/');
				const withoutLastPart = lastIndex !== -1 ? x.id.substring(0, lastIndex) : x.id;
				return { source: withoutLastPart, target: x.id, color: '#ff0000' };
			});

		const l = composition.concat(links, receivers);

		return {
			data: { nodes: n, links: l },
			linkStroke: (l: Link) => l.color,
			nodeLabel: (n: Node) => n.id.split('/').at(-1),
			nodeFill: (n: Node) => {
				switch (n.id.split('/').length) {
					case 1:
						return '#ff0000';
					case 2:
						return '#00ff00';
					case 3:
						return '#0000ff';
				}
			}
		};
	};

	const res = await compose();

	return res;
};
