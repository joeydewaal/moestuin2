import { createQuery, useQueryClient } from '@tanstack/svelte-query';
import { api, type Reading } from './api';

export const READINGS_KEY = ['readings'] as const;

export const readingsQuery = () =>
	createQuery<Reading[]>(() => ({
		queryKey: READINGS_KEY,
		queryFn: () => api.readings({ limit: 500 }),
		staleTime: 30_000
	}));

export function subscribeReadings(onReading: (r: Reading) => void): () => void {
	const es = new EventSource('/api/readings/stream', { withCredentials: true });
	es.onmessage = (ev) => {
		if (!ev.data) return;
		try {
			onReading(JSON.parse(ev.data) as Reading);
		} catch {
			// ignore
		}
	};
	return () => es.close();
}

export function useReadingsStream() {
	const qc = useQueryClient();
	return subscribeReadings((reading) => {
		qc.setQueryData<Reading[]>(READINGS_KEY, (prev) => {
			const next = [...(prev ?? []), reading];
			return next.length > 500 ? next.slice(next.length - 500) : next;
		});
	});
}
