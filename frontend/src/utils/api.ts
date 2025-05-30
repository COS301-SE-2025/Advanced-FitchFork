/**
 * Base URL used for all API requests.
 */
export const API_BASE_URL = import.meta.env.VITE_API_BASE_URL || 'http://localhost:3000/api';

/**
 * Standardized shape for all API responses.
 *
 * @template T - The expected type of the `data` payload.
 */
export interface ApiResponse<T> {
  /** Indicates if the request was successful. */
  success: boolean;

  /** The actual response data from the server. */
  data: T;

  /** A human-readable message from the server. */
  message: string;
}

/**
 * A wrapper around `fetch` for JSON-based API requests.
 *
 * - Automatically injects the auth token from `localStorage`.
 * - Handles token expiration.
 * - Parses and returns the API response as `ApiResponse<T>`.
 *
 * @template T - The expected shape of `data` in the response.
 * @param endpoint - API path or full URL.
 * @param options - `fetch` options like method, headers, body.
 */
export async function apiFetch<T>(
  endpoint: string,
  options: RequestInit = {}
): Promise<ApiResponse<T>> {
  // Construct full URL if only path is provided
  const url = endpoint.startsWith('http') ? endpoint : `${API_BASE_URL}${endpoint}`;

  // Load token and expiry from localStorage
  const stored = localStorage.getItem('auth');
  let token: string | null = null;
  let expires_at: string | null = null;

  if (stored) {
    try {
      const parsed = JSON.parse(stored);
      token = parsed?.token || null;
      expires_at = parsed?.expires_at || null;
    } catch {
      token = null;
    }
  }

  // If token is expired, clear it and return early
  if (expires_at && new Date(expires_at) < new Date()) {
    localStorage.removeItem('auth');
  }

  // Prepare headers including optional Authorization
  const headers: Record<string, string> = {
    'Content-Type': 'application/json',
    ...(options.headers as Record<string, string>),
  };

  if (token) {
    headers['Authorization'] = `Bearer ${token}`;
  }

  const finalOptions: RequestInit = {
    ...options,
    headers,
  };

  // Log outgoing request
  console.log('[apiFetch] →', {
    url,
    method: finalOptions.method || 'GET',
    headers: finalOptions.headers,
    body: finalOptions.body,
  });

  // Perform fetch request
  const res = await fetch(url, finalOptions);

  let data: ApiResponse<T>;

  // Attempt to parse JSON response
  try {
    data = await res.json();
  } catch (err) {
    console.error('[apiFetch] Failed to parse JSON response', err);
    throw new Error('Failed to parse response from server.');
  }

  // Log incoming response
  console.log('[apiFetch] ←', {
    status: res.status,
    ok: res.ok,
    data,
  });

  return data;
}


/**
 * Upload files using a FormData payload.
 *
 * @template T - The expected shape of `data` in the response.
 * @param endpoint - Upload endpoint (e.g. `/modules/:id/files`).
 * @param form - FormData containing files and/or fields.
 * @returns Parsed `ApiResponse<T>` or throws if invalid.
 */
export async function apiUpload<T>(
  endpoint: string,
  form: FormData
): Promise<ApiResponse<T>> {
  const url = endpoint.startsWith('http') ? endpoint : `${API_BASE_URL}${endpoint}`;

  const stored = localStorage.getItem('auth');
  let token: string | null = null;

  if (stored) {
    try {
      const parsed = JSON.parse(stored);
      token = parsed?.token || null;
    } catch {
      token = null;
    }
  }

  const res = await fetch(url, {
    method: 'POST',
    headers: token
      ? {
          Authorization: `Bearer ${token}`,
        }
      : undefined,
    body: form,
  });

  let data: ApiResponse<T>;

  try {
    data = await res.json();
  } catch {
    throw new Error('Failed to parse response from file upload.');
  }

  return data;
}

/**
 * Download a file as a blob and prompt the user to save it.
 *
 * @param endpoint - Download URL (e.g. `/modules/:id/files/:fileId`).
 */
export async function apiDownload(endpoint: string): Promise<void> {
  const url = endpoint.startsWith('http') ? endpoint : `${API_BASE_URL}${endpoint}`;

  const stored = localStorage.getItem('auth');
  let token: string | null = null;

  if (stored) {
    try {
      const parsed = JSON.parse(stored);
      token = parsed?.token || null;
    } catch {
      token = null;
    }
  }

  const headers: HeadersInit = {};
  if (token) headers['Authorization'] = `Bearer ${token}`;

  const res = await fetch(url, { method: 'GET', headers });

  if (!res.ok) {
    const fallback = await res.text();
    console.error('[apiDownload] Error:', fallback);
    throw new Error('Download failed');
  }

  const blob = await res.blob();

  const disposition = res.headers.get('Content-Disposition') || '';
  const filenameMatch = disposition.match(/filename="(.+?)"/);
  const rawFilename = filenameMatch?.[1] || 'downloaded_file';
  const decodedFilename = decodeURIComponent(rawFilename);

  const link = document.createElement('a');
  link.href = URL.createObjectURL(blob);
  link.download = decodedFilename;
  document.body.appendChild(link);
  link.click();
  document.body.removeChild(link);
  URL.revokeObjectURL(link.href);
}
