import { API_BASE_URL } from "@/config/api";
import type { ApiResponse } from "@/types/common";

/**
 * Serializes a flat or semi-structured object into a query string.
 * Special handling is included for sort arrays (e.g. SortOption[]).
 */
export const buildQuery = (params: Record<string, any>): string => {
  const query = new URLSearchParams();

  for (const key in params) {
    const value = params[key];

    if (value === undefined || value === null) continue;

    // Special case for sort: SortOption[]
    if (key === "sort" && Array.isArray(value)) {
      const sortString = value
        .map((s: { field: string; order: "ascend" | "descend" }) =>
          s.order === "descend" ? `-${s.field}` : s.field
        )
        .join(",");
      if (sortString) query.append("sort", sortString);
    }

    // Everything else: normal flat values
    else {
      query.append(key, value.toString());
    }
  }

  return query.toString();
};
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
// api.ts (or wherever this code lives)

export async function apiFetch<T>(
  endpoint: string,
  options: RequestInit = {}
): Promise<ApiResponse<T>> {
  const url = endpoint.startsWith('http') ? endpoint : `${API_BASE_URL}${endpoint}`;

  // Load token
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

  if (expires_at && new Date(expires_at) < new Date()) {
    localStorage.removeItem('auth');
  }

  // ---- HEADERS -------------------------------------------------------------
  // NOTE: If you sometimes send FormData through this function,
  // you may want to set Content-Type conditionally (not shown here).
  const headers: Record<string, string> = {
    'Content-Type': 'application/json',
    // 👇🏼 This header skips ngrok’s free-tier browser warning
    'ngrok-skip-browser-warning': 'true',
    ...(options.headers as Record<string, string>),
  };

  if (token) headers['Authorization'] = `Bearer ${token}`;

  const finalOptions: RequestInit = {
    ...options,
    headers,
    // 👇🏼 Optional: include cookies (needed for cookie-based auth/CORS)
    credentials: options.credentials ?? 'include',
  };

  console.log('[apiFetch] →', {
    url,
    method: finalOptions.method || 'GET',
    headers: finalOptions.headers,
    body: finalOptions.body,
  });

  const res = await fetch(url, finalOptions);
  const contentType = res.headers.get('content-type') || '';

  // ---- Handle ngrok interstitial / non-JSON --------------------------------
  // If ngrok injects its HTML page (ERR_NGROK_6024), content-type will be text/html
  // or missing. We'll parse text and convert to a clean ApiResponse error.
  if (!contentType.includes('application/json')) {
    const text = await res.text();

    // Heuristic: detect the ngrok warning page and make it explicit
    const looksLikeNgrokInterstitial =
      text.includes('cdn.ngrok.com/static/js/error.js') ||
      text.includes('ERR_NGROK_6024') ||
      text.includes('You are about to visit') ||
      text.includes('data-payload');

    const message = looksLikeNgrokInterstitial
      ? 'Request blocked by ngrok interstitial. Ensure header "ngrok-skip-browser-warning" is sent (added automatically).'
      : (text || 'Unknown non-JSON response');

    const data: ApiResponse<T> = {
      success: false,
      data: {} as T,
      message,
    };

    console.log('[apiFetch] ← (non-JSON)', { status: res.status, ok: res.ok, message });
    return data;
  }

  // ---- Normal JSON path ----------------------------------------------------
  try {
    const data = (await res.json()) as ApiResponse<T>;
    console.log('[apiFetch] ←', { status: res.status, ok: res.ok, data });
    return data;
  } catch (err) {
    console.error('[apiFetch] Failed to parse JSON', err);
    return {
      success: false,
      data: {} as T,
      message: 'Failed to parse response from server.',
    };
  }
}

// Returns the raw Blob without forcing a browser download.
// Keeps auth behavior consistent with apiFetch/apiDownload.
export async function apiFetchBlob(endpoint: string): Promise<Blob> {
  const url = endpoint.startsWith('http') ? endpoint : `${API_BASE_URL}${endpoint}`;

  const stored = localStorage.getItem('auth');
  let token: string | null = null;
  if (stored) {
    try { token = JSON.parse(stored)?.token || null; } catch { token = null; }
  }

  const headers: HeadersInit = { 'ngrok-skip-browser-warning': 'true' };
  if (token) headers['Authorization'] = `Bearer ${token}`;

  const res = await fetch(url, { method: 'GET', headers, credentials: 'include' });
  if (!res.ok) {
    // Try to extract API error details
    try {
      const j = await res.json();
      throw new Error(j?.message || res.statusText);
    } catch {
      const t = await res.text().catch(() => '');
      throw new Error(t || res.statusText || 'Download failed');
    }
  }
  return res.blob();
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


// ─────────────────────────────────────────────────────────────
// Lightweight HTTP verb helpers
// ─────────────────────────────────────────────────────────────
type QueryParams = Record<string, any>;
type RequestOptions = RequestInit & { params?: QueryParams; data?: unknown };

const withQuery = (endpoint: string, params?: QueryParams) => {
  if (!params || Object.keys(params).length === 0) return endpoint;
  const qs = buildQuery(params);
  if (!qs) return endpoint;
  return `${endpoint}${endpoint.includes('?') ? '&' : '?'}${qs}`;
};

export const api = {
  /** Escape hatch for any verb */
  request<T>(method: string, endpoint: string, opts: RequestOptions = {}) {
    const { params, data, ...init } = opts;
    const url = withQuery(endpoint, params);
    const options: RequestInit = { ...init, method };

    // Only set a body if caller didn't provide one via init.body
    if (data !== undefined && options.body === undefined) {
      // Allow passing raw bodies; otherwise JSON-stringify
      options.body =
        typeof data === 'string' ||
        data instanceof Blob ||
        data instanceof ArrayBuffer ||
        data instanceof FormData ||
        data instanceof URLSearchParams
          ? (data as any)
          : JSON.stringify(data);
    }

    return apiFetch<T>(url, options);
  },

  get<T>(endpoint: string, params?: QueryParams, init: RequestInit = {}) {
    return apiFetch<T>(withQuery(endpoint, params), { ...init, method: 'GET' });
  },

  head<T>(endpoint: string, params?: QueryParams, init: RequestInit = {}) {
    return apiFetch<T>(withQuery(endpoint, params), { ...init, method: 'HEAD' });
  },

  options<T>(endpoint: string, params?: QueryParams, init: RequestInit = {}) {
    return apiFetch<T>(withQuery(endpoint, params), { ...init, method: 'OPTIONS' });
  },

  post<T>(endpoint: string, data?: unknown, init: RequestInit = {}) {
    return this.request<T>('POST', endpoint, { ...init, data });
  },

  put<T>(endpoint: string, data?: unknown, init: RequestInit = {}) {
    return this.request<T>('PUT', endpoint, { ...init, data });
  },

  patch<T>(endpoint: string, data?: unknown, init: RequestInit = {}) {
    return this.request<T>('PATCH', endpoint, { ...init, data });
  },

  delete<T>(endpoint: string, data?: unknown, init: RequestInit = {}) {
    return this.request<T>('DELETE', endpoint, { ...init, data });
  },

  // optional alias
  del<T>(endpoint: string, data?: unknown, init: RequestInit = {}) {
    return this.delete<T>(endpoint, data, init);
  },
};
