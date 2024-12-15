import type { GraphInputNode } from '@unovis/ts';
import type { PageLoad } from './$types';

export const load: PageLoad = async ({ fetch }) => {
	const compose = async () => {
		const data = await fetch('/api/graphs');
		const graphs: string[] = await data.json();

		const receivers = (
			await Promise.all(
				graphs.map(async (graph) => {
					const data = await fetch(`/api/graphs/${graph}/receivers`);
					const json: string[][][] = await data.json();

					return json.map(([src, dst]) => {
						return [src.join('/'), `${graph}/${dst.join('/')}`];
					});
				})
			)
		)
			.flatMap((x) => x)

			.map(([a, b]) => ({ source: a, target: b }));

		const processors = (
			await Promise.all(
				graphs.map(async (graph) => {
					const data = await fetch(`/api/graphs/${graph}/nodes`);
					const json = (await data.json()) as string[];
					return json.map((node) => `${graph}/${node}`);
				})
			)
		).flatMap((x) => x);

		const links = (
			await Promise.all(
				graphs.map(async (graph) => {
					const data = await fetch(`/api/graphs/${graph}/links`);
					const json: string[][][] = await data.json();

					return json.map((x) => x.map((y) => `${graph}/${y.join('/')}`));
				})
			)
		)
			.flatMap((x) => x)
			.filter((x) => x.length > 1)
			.map(([a, b]) => ({ source: a, target: b }));

		const inputs = (
			await Promise.all(
				processors.map(async (n) => {
					const [graph, node] = n.split('/');
					const data = await fetch(`/api/graphs/${graph}/nodes/${node}/inputs`);
					const json = (await data.json()) as string[];
					return json.map((param) => `${graph}/${node}/${param}`);
				})
			)
		).flatMap((x) => x);

		const outputs = (
			await Promise.all(
				processors.map(async (n) => {
					const [graph, node] = n.split('/');
					const data = await fetch(`/api/graphs/${graph}/nodes/${node}/outputs`);
					const json = (await data.json()) as string[];
					return json.map((param) => `${graph}/${node}/${param}`);
				})
			)
		).flatMap((x) => x);

		const nodes = inputs.concat(outputs, processors, graphs);

		const n = nodes.map((x) => ({ id: x }));

		const l = nodes
			.filter((x) => x.includes('/'))
			.map((x) => {
				const lastIndex = x.lastIndexOf('/');
				const withoutLastPart = lastIndex !== -1 ? x.substring(0, lastIndex) : x;
				return { source: withoutLastPart, target: x };
			})
			.concat(links, receivers);

		return {
			data: { nodes: n, links: l },
			nodeLabel: (n: GraphInputNode) => (n.id as string).split('/').at(-1),
			nodeFill: (n: GraphInputNode) => {
				switch ((n.id as string).split('/').length) {
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
