// src/utils/api.ts

export interface ApiResponse<T> {
  success: boolean;
  data: T;
  message: string;
}

export const API_BASE_URL = 'http://127.0.0.1:3000/api';

export async function apiFetch<T>(
  endpoint: string,
  options: RequestInit = {}
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

  // console.log('[apiFetch] →', {
  //   url,
  //   method: finalOptions.method || 'GET',
  //   headers: finalOptions.headers,
  //   body: finalOptions.body,
  // });

  const res = await fetch(url, finalOptions);

  let data: ApiResponse<T>;

  try {
    data = await res.json();
  } catch (err) {
    console.error('[apiFetch] Failed to parse JSON response', err);
    throw new Error('Failed to parse response from server.');
  }

  // console.log('[apiFetch] ←', {
  //   status: res.status,
  //   ok: res.ok,
  //   data,
  // });

  return data;
}
