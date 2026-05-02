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

export interface Reading {
	id: string;
	taken_at: string;
	temp_c: number;
	humidity: number;
	moisture: number;
}

export interface WebcamLive {
	url: string | null;
	date: string | null;
}

export interface WebcamArchiveEntry {
	date: string;
	url: string;
}

export const api = {
	me: () => request<Me>('/auth/me'),
	readings: (params?: { from?: string; to?: string; limit?: number }) => {
		const q = new URLSearchParams();
		if (params?.from !== undefined) q.set('from', params.from);
		if (params?.to !== undefined) q.set('to', params.to);
		if (params?.limit !== undefined) q.set('limit', String(params.limit));
		const qs = q.toString();
		return request<Reading[]>(`/api/readings${qs ? `?${qs}` : ''}`);
	},
	webcamLive: () => request<WebcamLive>('/api/webcam/live'),
	webcamArchive: () => request<WebcamArchiveEntry[]>('/api/webcam/archive')
};
