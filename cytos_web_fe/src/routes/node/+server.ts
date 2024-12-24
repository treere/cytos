import { json, type RequestHandler } from '@sveltejs/kit';

export const POST: RequestHandler = async ({ request }) => {
	const data = await request.json();

	console.info(data);

	return json({ id: 'ok' }, { status: 200 });
};
