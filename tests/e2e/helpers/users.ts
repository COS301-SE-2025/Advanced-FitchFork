import type { APIRequestContext } from '@playwright/test';

export type TestUser = {
  id: number;
  username: string;
  email: string;
  admin: boolean;
  password: string;
};

type UpsertUserResponse = {
  success: boolean;
  data?: { id: number; username: string; email: string; admin: boolean };
  message?: string;
};

/**
 * Creates or updates a test user via `/api/test/users`.
 * Uses the provided Playwright `api` fixture.
 */
export async function createUser(
  api: APIRequestContext,
  user: { username: string; password: string; email?: string; admin?: boolean }
): Promise<TestUser> {
  const email = user.email ?? `${user.username}@example.com`;

  const res = await api.post(`/api/test/users`, {
    data: { ...user, email },
  });

  if (!res.ok()) {
    throw new Error(`createUser failed ${res.status()}: ${await res.text()}`);
  }

  const json = (await res.json()) as UpsertUserResponse;
  if (!json.success || !json.data?.id) {
    throw new Error(`createUser error: ${json.message ?? 'unknown'}`);
  }

  return {
    id: json.data.id,
    username: json.data.username,
    email: json.data.email,
    admin: !!json.data.admin,
    password: user.password,
  };
}

/**
 * Deletes a test user by numeric ID via `/api/test/users/{id}`.
 * Uses the provided Playwright `api` fixture.
 */
export async function deleteUser(api: APIRequestContext, userId: number): Promise<boolean> {
  const res = await api.delete(`/api/test/users/${userId}`);

  if (res.ok()) {
    return true;
  } else {
    console.warn(`Could not delete test user id ${userId} (status ${res.status()})`);
    return false;
  }
}
