import { API_BASE_URL } from "@/config/api";
import type { ApiResponse } from "@/types/common";
import { getAssignmentPin, clearAssignmentPin } from "@/utils/assignmentAccess";

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
    } else {
      query.append(key, value.toString());
    }
  }

  return query.toString();
};

/** Extract /modules/:mid/assignments/:aid from a (possibly absolute) URL. */
const matchAssignmentIds = (fullUrl: string): { moduleId: number; assignmentId: number } | null => {
  try {
    // Strip origin if present
    const u = fullUrl.startsWith("http") ? new URL(fullUrl).pathname : fullUrl;
    const m = u.match(/\/modules\/(\d+)\/assignments\/(\d+)/);
    if (!m) return null;
    return { moduleId: Number(m[1]), assignmentId: Number(m[2]) };
  } catch {
    return null;
  }
};

/** Adds auth + ngrok header + (if applicable) x-assignment-pin. */
const buildHeaders = (base: HeadersInit | undefined, url: string, token: string | null): Record<string, string> => {
  const headers: Record<string, string> = {
    "Content-Type": "application/json",
    "ngrok-skip-browser-warning": "true",
    ...(base as Record<string, string>),
  };
  if (token) headers["Authorization"] = `Bearer ${token}`;

  const ids = matchAssignmentIds(url);
  if (ids) {
    const pin = getAssignmentPin(ids.moduleId, ids.assignmentId);
    if (pin) headers["x-assignment-pin"] = pin;
  }
  return headers;
};

/** Redacts sensitive headers for logging. */
const redactHeadersForLog = (headers: Record<string, any>) => {
  const redacted: Record<string, any> = { ...headers };
  if (redacted["x-assignment-pin"]) redacted["x-assignment-pin"] = "***";
  if (redacted["Authorization"]) redacted["Authorization"] = "Bearer ***";
  return redacted;
};

/** Should we log sensitive fields (full headers/body) without redaction? */
const shouldLogSensitive = (): boolean => {
  try {
    // Enable with: localStorage.LOG_SENSITIVE = "1" or window.__LOG_SENSITIVE__ = true
    // @ts-ignore
    if (typeof window !== "undefined" && (window.__LOG_SENSITIVE__ === true)) return true;
    if (typeof window !== "undefined" && localStorage.getItem("LOG_SENSITIVE") === "1") return true;
  } catch { /* noop */ }
  return false;
};

/** Make a plain object from Headers (Response headers). */
const headersToObject = (h: Headers): Record<string, string> => {
  const obj: Record<string, string> = {};
  try {
    h.forEach((v, k) => { obj[k] = v; });
  } catch { /* noop */ }
  return obj;
};

/** Ensure request body is printable. */
const bodyPreviewForLog = (body: unknown) => {
  try {
    if (typeof body === "string") {
      return body.length > 10000 ? `${body.slice(0, 10000)}… [truncated]` : body;
    }
    if (body instanceof URLSearchParams) return body.toString();
    if (body instanceof FormData) {
      const entries: Record<string, any> = {};
      (body as FormData).forEach((v, k) => {
        entries[k] = v instanceof Blob ? `[Blob ${v.type} ${v.size}B]` : v;
      });
      return entries;
    }
    if (body instanceof Blob) return `[Blob ${body.type} ${body.size}B]`;
    if (body instanceof ArrayBuffer) return `[ArrayBuffer ${body.byteLength}B]`;
    if (typeof body === "object" && body !== null) return JSON.stringify(body);
  } catch { /* noop */ }
  return body as any;
};

/** If server says the PIN is invalid, clear it so the user gets re-prompted. */
const maybeClearPinOnForbidden = async (res: Response, url: string) => {
  if (res.status !== 403) return;
  let msg = "";
  try {
    const clone = res.clone();
    const ct = clone.headers.get("content-type") || "";
    if (ct.includes("application/json")) {
      const j = await clone.json().catch(() => null);
      msg = j?.message || "";
    } else {
      msg = await clone.text().catch(() => "");
    }
  } catch { /* noop */ }

  // Backend messages to watch for
  const looksLikePinIssue =
    /password required|invalid pin|invalid password|assignment password/i.test(msg);

  if (looksLikePinIssue) {
    const ids = matchAssignmentIds(url);
    if (ids) clearAssignmentPin(ids.moduleId, ids.assignmentId);
  }
};

/**
 * A wrapper around `fetch` for JSON-based API requests.
 *
 * - Automatically injects the auth token from `localStorage`.
 * - Handles token expiration.
 * - Injects `x-assignment-pin` for assignment endpoints (if present in sessionStorage).
 * - Parses and returns the API response as `ApiResponse<T>`.
 * - Logs the full request & full response (redacted by default; see shouldLogSensitive()).
 */
export async function apiFetch<T>(
  endpoint: string,
  options: RequestInit = {}
): Promise<ApiResponse<T>> {
  const url = endpoint.startsWith("http") ? endpoint : `${API_BASE_URL}${endpoint}`;

  // Load token
  const stored = localStorage.getItem("auth");
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
    localStorage.removeItem("auth");
  }

  const headers = buildHeaders(options.headers, url, token);

  const finalOptions: RequestInit = {
    ...options,
    headers,
    credentials: options.credentials ?? "include",
  };

  // Print full REQUEST
  const reqLog = {
    url,
    method: finalOptions.method || "GET",
    credentials: finalOptions.credentials,
    headers: shouldLogSensitive()
      ? (finalOptions.headers as Record<string, any>)
      : redactHeadersForLog(finalOptions.headers as Record<string, any>),
    body: bodyPreviewForLog(finalOptions.body),
  };
  console.log("[apiFetch] → REQUEST", reqLog);

  // Do fetch
  const res = await fetch(url, finalOptions);
  await maybeClearPinOnForbidden(res, url);

  // Clone to read body for logs without consuming stream
  const resClone = res.clone();
  const contentType = res.headers.get("content-type") || "";
  const respHeadersObj = headersToObject(res.headers);

  // Print full RESPONSE (non-JSON)
  if (!contentType.includes("application/json")) {
    const text = await resClone.text().catch(() => "");
    const looksLikeNgrokInterstitial =
      text.includes("cdn.ngrok.com/static/js/error.js") ||
      text.includes("ERR_NGROK_6024") ||
      text.includes("You are about to visit") ||
      text.includes("data-payload");

    const message = looksLikeNgrokInterstitial
      ? 'Request blocked by ngrok interstitial. Ensure header "ngrok-skip-browser-warning" is sent (added automatically).'
      : (text || "Unknown non-JSON response");

    const data: ApiResponse<T> = {
      success: false,
      data: {} as T,
      message,
    };

    console.log("[apiFetch] ← RESPONSE (non-JSON)", {
      status: res.status,
      ok: res.ok,
      headers: respHeadersObj,
      body: text,
    });

    return data;
  }

  // JSON path
  try {
    // Read the clone to log full JSON body
    const bodyText = await resClone.text();
    let parsedForLog: any = bodyText;
    try {
      parsedForLog = JSON.parse(bodyText);
    } catch { /* keep as text if not valid JSON */ }

    console.log("[apiFetch] ← RESPONSE", {
      status: res.status,
      ok: res.ok,
      headers: respHeadersObj,
      body: parsedForLog,
    });

    // Now parse the actual response to return
    const data = (await res.json()) as ApiResponse<T>;
    return data;
  } catch (err) {
    console.error("[apiFetch] Failed to parse JSON", err);
    return {
      success: false,
      data: {} as T,
      message: "Failed to parse response from server.",
    };
  }
}

// Returns the raw Blob without forcing a browser download.
// Keeps auth behavior consistent with apiFetch/apiDownload.
export async function apiFetchBlob(endpoint: string): Promise<Blob> {
  const url = endpoint.startsWith("http") ? endpoint : `${API_BASE_URL}${endpoint}`;

  const stored = localStorage.getItem("auth");
  let token: string | null = null;
  if (stored) {
    try { token = JSON.parse(stored)?.token || null; } catch { token = null; }
  }

  const headers = buildHeaders({} as any, url, token);

  console.log("[apiFetchBlob] → REQUEST", {
    url,
    method: "GET",
    headers: shouldLogSensitive() ? headers : redactHeadersForLog(headers),
  });

  const res = await fetch(url, { method: "GET", headers, credentials: "include" });
  await maybeClearPinOnForbidden(res, url);

  const respHeadersObj = headersToObject(res.headers);

  if (!res.ok) {
    // Try to extract API error details
    try {
      const j = await res.clone().json();
      console.log("[apiFetchBlob] ← RESPONSE (error JSON)", {
        status: res.status,
        ok: res.ok,
        headers: respHeadersObj,
        body: j,
      });
      throw new Error(j?.message || res.statusText);
    } catch {
      const t = await res.clone().text().catch(() => "");
      console.log("[apiFetchBlob] ← RESPONSE (error text)", {
        status: res.status,
        ok: res.ok,
        headers: respHeadersObj,
        body: t,
      });
      throw new Error(t || res.statusText || "Download failed");
    }
  }

  const blob = await res.clone().blob();
  console.log("[apiFetchBlob] ← RESPONSE (blob)", {
    status: res.status,
    ok: res.ok,
    headers: respHeadersObj,
    size: blob.size,
    type: blob.type,
  });

  return res.blob();
}

/**
 * Upload files using a FormData payload.
 */
export async function apiUpload<T>(
  endpoint: string,
  form: FormData
): Promise<ApiResponse<T>> {
  const url = endpoint.startsWith("http") ? endpoint : `${API_BASE_URL}${endpoint}`;

  const stored = localStorage.getItem("auth");
  let token: string | null = null;

  if (stored) {
    try {
      const parsed = JSON.parse(stored);
      token = parsed?.token || null;
    } catch {
      token = null;
    }
  }

  // NOTE: Do not set Content-Type for FormData; browser sets boundaries.
  const headers: HeadersInit = token ? { Authorization: `Bearer ${token}` } : {};
  // Also attach PIN if applicable
  const ids = matchAssignmentIds(url);
  if (ids) {
    const pin = getAssignmentPin(ids.moduleId, ids.assignmentId);
    if (pin) (headers as any)["x-assignment-pin"] = pin;
  }

  // Log request with a readable FormData preview
  const formPreview: Record<string, any> = {};
  try {
    form.forEach((v, k) => {
      formPreview[k] = v instanceof Blob ? `[Blob ${v.type} ${v.size}B]` : v;
    });
  } catch { /* noop */ }

  console.log("[apiUpload] → REQUEST", {
    url,
    method: "POST",
    headers: shouldLogSensitive() ? headers : redactHeadersForLog(headers as any),
    form: formPreview,
  });

  const res = await fetch(url, {
    method: "POST",
    headers,
    body: form,
    credentials: "include",
  });

  await maybeClearPinOnForbidden(res, url);

  const respHeadersObj = headersToObject(res.headers);

  let data: ApiResponse<T>;
  try {
    const text = await res.clone().text();
    let parsed: any = text;
    try { parsed = JSON.parse(text); } catch { /* keep text */ }

    console.log("[apiUpload] ← RESPONSE", {
      status: res.status,
      ok: res.ok,
      headers: respHeadersObj,
      body: parsed,
    });

    data = await res.json();
  } catch {
    console.error("[apiUpload] Failed to parse response from file upload.");
    throw new Error("Failed to parse response from file upload.");
  }
  return data;
}

/**
 * Download a file as a blob and prompt the user to save it.
 */
export async function apiDownload(endpoint: string): Promise<void> {
  const url = endpoint.startsWith("http") ? endpoint : `${API_BASE_URL}${endpoint}`;

  const stored = localStorage.getItem("auth");
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
  if (token) headers["Authorization"] = `Bearer ${token}`;

  // attach PIN if applicable
  const ids = matchAssignmentIds(url);
  if (ids) {
    const pin = getAssignmentPin(ids.moduleId, ids.assignmentId);
    if (pin) (headers as any)["x-assignment-pin"] = pin;
  }

  console.log("[apiDownload] → REQUEST", {
    url,
    method: "GET",
    headers: shouldLogSensitive() ? headers : redactHeadersForLog(headers as any),
  });

  const res = await fetch(url, { method: "GET", headers, credentials: "include" });
  await maybeClearPinOnForbidden(res, url);

  const respHeadersObj = headersToObject(res.headers);

  if (!res.ok) {
    const fallback = await res.clone().text();
    console.log("[apiDownload] ← RESPONSE (error)", {
      status: res.status,
      ok: res.ok,
      headers: respHeadersObj,
      body: fallback,
    });
    throw new Error("Download failed");
  }

  const blob = await res.clone().blob();

  console.log("[apiDownload] ← RESPONSE (blob)", {
    status: res.status,
    ok: res.ok,
    headers: respHeadersObj,
    size: blob.size,
    type: blob.type,
  });

  const disposition = res.headers.get("Content-Disposition") || "";
  const filenameMatch = disposition.match(/filename="(.+?)"/);
  const rawFilename = filenameMatch?.[1] || "downloaded_file";
  const decodedFilename = decodeURIComponent(rawFilename);

  const link = document.createElement("a");
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
  return `${endpoint}${endpoint.includes("?") ? "&" : "?"}${qs}`;
};

export const api = {
  request<T>(method: string, endpoint: string, opts: RequestOptions = {}) {
    const { params, data, ...init } = opts;
    const url = withQuery(endpoint, params);
    const options: RequestInit = { ...init, method };

    if (data !== undefined && options.body === undefined) {
      options.body =
        typeof data === "string" ||
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
    return apiFetch<T>(withQuery(endpoint, params), { ...init, method: "GET" });
  },

  head<T>(endpoint: string, params?: QueryParams, init: RequestInit = {}) {
    return apiFetch<T>(withQuery(endpoint, params), { ...init, method: "HEAD" });
  },

  options<T>(endpoint: string, params?: QueryParams, init: RequestInit = {}) {
    return apiFetch<T>(withQuery(endpoint, params), { ...init, method: "OPTIONS" });
  },

  post<T>(endpoint: string, data?: unknown, init: RequestInit = {}) {
    return this.request<T>("POST", endpoint, { ...init, data });
  },

  put<T>(endpoint: string, data?: unknown, init: RequestInit = {}) {
    return this.request<T>("PUT", endpoint, { ...init, data });
  },

  patch<T>(endpoint: string, data?: unknown, init: RequestInit = {}) {
    return this.request<T>("PATCH", endpoint, { ...init, data });
  },

  delete<T>(endpoint: string, data?: unknown, init: RequestInit = {}) {
    return this.request<T>("DELETE", endpoint, { ...init, data });
  },

  del<T>(endpoint: string, data?: unknown, init: RequestInit = {}) {
    return this.delete<T>(endpoint, data, init);
  },
};
