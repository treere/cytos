import { invalidate, invalidateAll } from '$app/navigation';

export const linkNodes = async (source: string, target: string) => {
	const s = source.split('/').filter((x) => x);
	const t = target.split('/').filter((x) => x);
	console.info('>', s, t);
	if (s.length != 3 || t.length != 3) return;
	console.info('here');
	const [s_g, s_n, s_p] = s;
	const [t_g, t_n, t_p] = t;

	if (s_g != t_g) return;
	await fetch(`/api/graphs/${s_g}/nodes/link`, {
		method: 'POST',
		body: JSON.stringify([
			[s_n, s_p],
			[t_n, t_p]
		]),
		headers: {
			'Content-Type': 'application/json'
		}
	});
	invalidateAll();
};

export const graphStart = async (graph: string) => {
	await fetch(`/api/graphs/${graph}/start`, { method: 'POST' });
	invalidate(`/api/graphs/${graph}`);
};

export const graphStop = async (graph: string) => {
	await fetch(`/api/graphs/${graph}/stop`, { method: 'POST' });
	invalidate(`/api/graphs/${graph}`);
};
