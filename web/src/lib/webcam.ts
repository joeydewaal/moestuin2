import { createQuery } from '@tanstack/svelte-query';
import { api, type WebcamArchiveEntry, type WebcamLive } from './api';

export const WEBCAM_LIVE_KEY = ['webcam', 'live'] as const;
export const WEBCAM_ARCHIVE_KEY = ['webcam', 'archive'] as const;

export const liveQuery = () =>
	createQuery<WebcamLive>(() => ({
		queryKey: WEBCAM_LIVE_KEY,
		queryFn: () => api.webcamLive(),
		staleTime: 30_000,
		// Pick up day rollover without forcing a manual refresh.
		refetchInterval: 5 * 60_000
	}));

export const archiveQuery = () =>
	createQuery<WebcamArchiveEntry[]>(() => ({
		queryKey: WEBCAM_ARCHIVE_KEY,
		queryFn: () => api.webcamArchive(),
		staleTime: 5 * 60_000
	}));
