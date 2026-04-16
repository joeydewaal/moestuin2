export interface Me {
	id: string;
	email: string;
	name: string | null;
}

export class UnauthorizedError extends Error {
	constructor() {
		super('unauthorized');
	}
}

async function request<T>(path: string, init?: RequestInit): Promise<T> {
	const res = await fetch(path, {
		credentials: 'include',
		...init,
		headers: { Accept: 'application/json', ...(init?.headers ?? {}) }
	});
	if (res.status === 401) throw new UnauthorizedError();
	if (!res.ok) throw new Error(`${res.status} ${res.statusText}`);
	return res.json() as Promise<T>;
}

export const api = {
	me: () => request<Me>('/auth/me')
};
