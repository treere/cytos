import type { PageLoad } from './$types';

const zip = (...arrays: any[]) => {
	const maxLength = Math.max(...arrays.map((x) => x.length));
	return Array.from({ length: maxLength }).map((_, i) => {
		return Array.from({ length: arrays.length }, (_, k) => arrays[k][i]);
	});
};

export const load: PageLoad = async ({ fetch }) => {
	const compose = async () => {
		const graphNames: string[] = await fetch('/api/graphs').then((x) => x.json());

		let nodes: string[][] = await Promise.all(
			graphNames.map((graph) => fetch(`/api/graphs/${graph}/nodes`).then((x) => x.json()))
		);

		let allInternalLinks: string[][] = await Promise.all(
			graphNames.map((graph) => fetch(`/api/graphs/${graph}/links`).then((x) => x.json()))
		);

		const panels = zip(graphNames, nodes).map(([g, n]) => ({ nodes: n, label: g }));

		return {
			panels,
			data: { nodes: nodes.flatMap((n) => n).map((n) => ({ id: n })), links: [] },
			allInternalLinks
		};
	};

	const res = await compose();
	console.info(res);
	return res;
};
