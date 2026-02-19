import type { Handle } from '@sveltejs/kit';

export const handle: Handle = async ({ event, resolve }) => {
	if (event.url.pathname.startsWith('/api')) {
		return fetch(`http://localhost:3000${event.url.pathname.replace(/^\/api/, '')}`, event.request);
	}

	const response = await resolve(event);
	return response;
};
