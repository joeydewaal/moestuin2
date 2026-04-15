import { describe, it, expect } from 'vitest';
import { UnauthorizedError } from './api';

describe('UnauthorizedError', () => {
	it('carries the "unauthorized" message', () => {
		expect(new UnauthorizedError().message).toBe('unauthorized');
	});
});
