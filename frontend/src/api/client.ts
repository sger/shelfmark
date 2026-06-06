const API_BASE = '/api/v1';
let authToken: string | null = null;

export function setAuthToken(token: string | null) {
  authToken = token;
}

type ApiOptions = Omit<RequestInit, 'body'> & {
  body?: unknown;
};

export async function api<T>(path: string, options: ApiOptions = {}): Promise<T> {
  const headers = new Headers(options.headers);
  if (!(options.body instanceof FormData)) {
    headers.set('Content-Type', 'application/json');
  }
  if (authToken) {
    headers.set('Authorization', `Bearer ${authToken}`);
  }

  const response = await fetch(`${API_BASE}${path}`, {
    ...options,
    headers,
    body: options.body instanceof FormData ? options.body : options.body ? JSON.stringify(options.body) : undefined,
  });

  if (!response.ok) {
    const payload = await response.json().catch(() => ({ error: response.statusText }));
    throw new Error(payload.error ?? `${response.status} ${response.statusText}`);
  }

  return response.json() as Promise<T>;
}

export function fileUrl(bookId: string) {
  return `${API_BASE}/books/${bookId}/file`;
}

export function authHeaders(): Record<string, string> {
  return authToken ? { Authorization: `Bearer ${authToken}` } : {};
}
