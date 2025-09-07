import fs from 'node:fs';
import path from 'node:path';
import type { FullConfig } from '@playwright/test';
import { request } from '@playwright/test';
import { AUTH_DIR } from '@helpers/constants';

type UpsertUserResponse = {
  success: boolean;
  data?: { id: number; username: string; email: string; admin: boolean };
  message?: string;
};

async function upsertUser(
  api: any,
  apiURL: string,
  user: { username: string; email: string; password: string; admin?: boolean }
) {
  const res = await api.post(`${apiURL}/api/test/users`, { data: user });
  if (!res.ok()) {
    throw new Error(`upsertUser failed ${res.status()}: ${await res.text()}`);
  }

  const json = (await res.json()) as UpsertUserResponse;
  if (!json.success || !json.data?.id) {
    throw new Error(`Invalid upsert response: ${json.message ?? 'unknown'}`);
  }

  return { ...json.data, password: user.password };
}

async function loginUser(api: any, apiURL: string, username: string, password: string) {
  const res = await api.post(`${apiURL}/api/auth/login`, { data: { username, password } });
  if (!res.ok()) throw new Error(`login failed ${res.status()}`);
  return (await res.json()).data;
}

export default async function globalSetup(config: FullConfig) {
  // Ensure auth directory exists
  fs.mkdirSync(AUTH_DIR, { recursive: true });

  const baseURL = config.projects[0].use!.baseURL as string;
  const apiURL = process.env.E2E_API_URL ?? 'http://localhost:3000';
  const password = process.env.E2E_TEST_USER_PASSWORD ?? '1';

  const api = await request.newContext();

  const userDefs = [
    { username: 'admin',     email: 'admin@example.com',     admin: true },
    { username: 'lecturer',  email: 'lecturer@example.com',  admin: false },
    { username: 'assistant', email: 'assistant@example.com', admin: false },
    { username: 'tutor',     email: 'tutor@example.com',     admin: false },
    { username: 'student',   email: 'student@example.com',   admin: false },
  ];

  for (const def of userDefs) {
    const userInfo = await upsertUser(api, apiURL, { ...def, password });
    const { token, expires_at } = await loginUser(api, apiURL, def.username, password);

    const storageState = {
      cookies: [],
      origins: [
        {
          origin: new URL(baseURL).origin,
          localStorage: [
            {
              name: 'auth',
              value: JSON.stringify({ token, expires_at, user: null, modules: [] }),
            },
          ],
        },
      ],
      __meta: { id: userInfo.id, username: userInfo.username },
    };

    fs.writeFileSync(path.join(AUTH_DIR, `${def.username}.json`), JSON.stringify(storageState, null, 2));
  }

  await api.dispose();
}
