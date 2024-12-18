import { invalidate, invalidateAll } from '$app/navigation';

const decodePath = (path: string) => {
	const p = path.split('/').filter((x) => x);
	if (p.length != 3) throw new Error('invalid path');
	return { graph: p[0], node: p[1], prop: p[2] };
};

export const linkNodes = async (source: string, target: string) => {
	const s = decodePath(source);
	const t = decodePath(target);

	if (s.graph != t.graph) throw new Error('conflicting graphs');

	await fetch(`/api/graphs/${s.graph}/nodes/link`, {
		method: 'POST',
		body: JSON.stringify([
			[s.node, s.prop],
			[t.node, t.prop]
		]),
		headers: {
			'Content-Type': 'application/json'
		}
	});
	invalidate(`/api/graphs/${s.graph}/links`);
};

export const graphStart = async (graph: string) => {
	await fetch(`/api/graphs/${graph}/start`, { method: 'POST' });
	invalidate(`/api/graphs/${graph}`);
};

export const graphStop = async (graph: string) => {
	await fetch(`/api/graphs/${graph}/stop`, { method: 'POST' });
	invalidate(`/api/graphs/${graph}`);
};
