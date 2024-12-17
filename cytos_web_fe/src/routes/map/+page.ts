import type { PageLoad } from './$types';

export interface Link {
	source: string;
	target: string;
	color: string;
	active?: boolean;
}

enum NodeType {
	Graph = 'Graph',
	Node = 'Node',
	InputParam = 'Input',
	OutputParam = 'Output'
}

export interface Node {
	id: string;
	color: string;
	label: string;
	type: NodeType;
	link: string;
}

export const load: PageLoad = async ({ fetch }) => {
	const create_graph = async () => {
		const data = await fetch('/api/graphs');
		const graphs: Node[] = (await data.json()).map((x: string) => ({
			id: x,
			color: '#ff0000',
			label: x,
			type: NodeType.Graph,
			link: `/graphs/${x}`
		}));

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
			.map(([a, b]) => ({ source: a, target: b, color: '#0000ff', active: true }));

		const processorsData: Node[][] = await Promise.all(
			graphs.map(async (graph) => {
				const data = await fetch(`/api/graphs/${graph.id}/nodes`);
				const json = (await data.json()) as string[];
				return json.map((node) => ({
					id: `${graph.id}/${node}`,
					color: '#00ff00',
					label: node,
					type: NodeType.Node,
					link: `/graphs/${graph.id}/nodes/${node}`
				}));
			})
		);

		const processors = processorsData.flatMap((x) => x);

		const processorsLinks = processorsData.flatMap((group) => {
			const links: Link[] = [];
			if (group.length < 2) return [];
			for (let i = 0; i < group.length - 1; i += 1) {
				const [a, b] = group.slice(i, i + 2);
				links.push({ source: a.id, target: b.id, color: '#00ffff', active: true });
			}
			return links;
		});

		const linkGroups: string[][] = (
			await Promise.all(
				graphs.map(async (graph) => {
					const data = await fetch(`/api/graphs/${graph.id}/links`);
					const json: string[][][] = await data.json();

					return json.map((x) => x.map((y) => `${graph.id}/${y.join('/')}`));
				})
			)
		)
			.flatMap((x) => x)
			.filter((x) => x.length > 1);

		const links = linkGroups.flatMap((group) => {
			const links: Link[] = [];
			for (let i = 0; i < group.length - 1; i += 1) {
				const [a, b] = group.slice(i, i + 2);
				links.push({ source: a, target: b, color: '#00ff00', active: false });
			}
			return links;
		});

		const inputs: Node[] = (
			await Promise.all(
				processors.map(async (n) => {
					const [graph, node] = n.id.split('/');
					const data = await fetch(`/api/graphs/${graph}/nodes/${node}/inputs`);
					const json = (await data.json()) as string[];
					return json.map((param) => ({
						id: `${graph}/${node}/${param}`,
						color: '#0000ff',
						label: param,
						type: NodeType.InputParam,
						link: `/graphs/${graph}/nodes/${node}/params/${param}`
					}));
				})
			)
		).flatMap((x) => x);

		const outputs: Node[] = (
			await Promise.all(
				processors.map(async (n) => {
					const [graph, node] = n.id.split('/');
					const data = await fetch(`/api/graphs/${graph}/nodes/${node}/outputs`);
					const json = (await data.json()) as string[];
					return json.map((param) => ({
						id: `${graph}/${node}/${param}`,
						color: '#0000ff',
						label: param,
						type: NodeType.OutputParam,
						link: `/graphs/${graph}/nodes/${node}/params/${param}`
					}));
				})
			)
		).flatMap((x) => x);

		const n = graphs.concat(inputs, outputs, processors);

		const composition: Link[] = n
			.filter((x) => x.id.includes('/'))
			.map((x) => {
				const lastIndex = x.id.lastIndexOf('/');
				const withoutLastPart = lastIndex !== -1 ? x.id.substring(0, lastIndex) : x.id;
				const source = x.type == NodeType.OutputParam ? withoutLastPart : x.id;
				const target = x.type == NodeType.OutputParam ? x.id : withoutLastPart;
				return {
					source,
					target,
					color: '#ff0000',
					active: !(x.type in [NodeType.InputParam, NodeType.OutputParam])
				};
			});

		const l = composition.concat(links, receivers, processorsLinks);

		return { nodes: n, links: l };
	};

	return {
		graph: create_graph(),
		linkStroke: (l: Link) => l.color,
		nodeLabel: (n: Node) => n.label,
		nodeFill: (n: Node) => n.color,
		linkFlow: (l: Link) => l.active
	};
};
