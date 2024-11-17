import type { PageLoad } from './$types';

export const load: PageLoad = async ({ fetch }) => {
	return {
		graphs: fetch('/api/graphs').then((x) => x.json())
	};
};
