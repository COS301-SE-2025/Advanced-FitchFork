import type { APIRequestContext } from '@playwright/test';

export type LoginResponse = {
  success: boolean;
  data?: {
    id: number;
    username: string;
    email: string;
    admin: boolean;
    token: string;
    expires_at: string;
  };
  message?: string;
};

export async function login(
  api: APIRequestContext,
  username: string,
  password: string
): Promise<LoginResponse> {
  const res = await api.post('/api/auth/login', { data: { username, password } });
  const body = (await res.json()) as LoginResponse;
  if (!res.ok() || !body.success || !body.data?.token) {
    throw new Error(`Login failed for ${username}: ${res.status()} ${JSON.stringify(body)}`);
  }
  return body;
}
