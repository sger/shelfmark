import { describe, expect, it } from 'vitest';
import { authHeaders, fileUrl, setAuthToken } from './client';

describe('api client helpers', () => {
  it('builds book file URLs under the versioned API', () => {
    expect(fileUrl('book-1')).toBe('/api/v1/books/book-1/file');
  });

  it('returns bearer headers only when a token is set', () => {
    setAuthToken(null);
    expect(authHeaders()).toEqual({});
    setAuthToken('token-1');
    expect(authHeaders()).toEqual({ Authorization: 'Bearer token-1' });
    setAuthToken(null);
  });
});

