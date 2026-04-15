import { createQuery } from '@tanstack/svelte-query';
import { api, UnauthorizedError, type Me } from './api';

export const SESSION_KEY = ['session'] as const;

export const sessionQuery = () =>
	createQuery<Me | null>(() => ({
		queryKey: SESSION_KEY,
		queryFn: async () => {
			try {
				return await api.me();
			} catch (err) {
				if (err instanceof UnauthorizedError) return null;
				throw err;
			}
		},
		staleTime: 60_000,
		retry: false
	}));
