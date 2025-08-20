import type { APIRequestContext } from '@playwright/test';
import { login } from '@helpers/auth';

export type ModuleInput = {
  code: string;
  year: number;
  description?: string;
  credits: number;
};

export type ModuleRecord = {
  id: number;
  code: string;
  year: number;
  description: string | null;
  credits: number;
  created_at?: string;
  updated_at?: string;
};

type ApiEnvelope<T> = {
  success: boolean;
  data?: T;
  message?: string;
};

/**
 * Low-level: create a module with an auth token.
 * Throws on any non-2xx or { success:false } response.
 */
export async function createModule(
  api: APIRequestContext,
  body: ModuleInput,
  token: string
): Promise<ModuleRecord> {
  const res = await api.post('/api/modules', {
    data: body,
    headers: { Authorization: `Bearer ${token}` },
  });

  // Surface useful error text for 400/403/409/500 etc.
  if (!res.ok()) {
    throw new Error(`createModule failed ${res.status()}: ${await res.text()}`);
  }

  const json = (await res.json()) as ApiEnvelope<ModuleRecord>;
  if (!json.success || !json.data) {
    throw new Error(`createModule error: ${json.message ?? 'unknown error'}`);
  }
  return json.data;
}

/**
 * Low-level: delete a module by ID with an auth token.
 * Throws on any non-2xx or { success:false } response.
 */
export async function deleteModule(
  api: APIRequestContext,
  moduleId: number | string,
  token: string
): Promise<void> {
  const res = await api.delete(`/api/modules/${moduleId}`, {
    headers: { Authorization: `Bearer ${token}` },
  });

  if (!res.ok()) {
    throw new Error(`deleteModule(${moduleId}) failed ${res.status()}: ${await res.text()}`);
  }

  const json = (await res.json()) as ApiEnvelope<null>;
  if (!json.success) {
    throw new Error(`deleteModule(${moduleId}) error: ${json.message ?? 'unknown error'}`);
  }
}

/**
 * Convenience: create a module as the admin user.
 * Uses E2E_TEST_USER_PASSWORD (defaults to "1").
 */
export async function createModuleAsAdmin(
  api: APIRequestContext,
  body: ModuleInput,
  adminUser = 'admin',
  adminPassword = process.env.E2E_TEST_USER_PASSWORD ?? '1'
): Promise<ModuleRecord> {
  const auth = await login(api, adminUser, adminPassword);
  const token = auth.data!.token;
  return createModule(api, body, token);
}

/**
 * Convenience: delete a module as the admin user.
 * Uses E2E_TEST_USER_PASSWORD (defaults to "1").
 */
export async function deleteModuleAsAdmin(
  api: APIRequestContext,
  moduleId: number | string,
  adminUser = 'admin',
  adminPassword = process.env.E2E_TEST_USER_PASSWORD ?? '1'
): Promise<void> {
  const auth = await login(api, adminUser, adminPassword);
  const token = auth.data!.token;
  return deleteModule(api, moduleId, token);
}
