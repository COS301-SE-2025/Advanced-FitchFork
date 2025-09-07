import { test as base, request, type APIRequestContext, type Page, type Browser } from '@playwright/test';
import path from 'node:path';
import fs from 'node:fs';
import { AUTH_DIR } from '@helpers/constants';

type Fixtures = {
  api: APIRequestContext;
  as: (username: string) => Promise<Page>;
};

// --- helpers ---------------------------------------------------------------

const API_URL = process.env.E2E_API_URL ?? 'http://localhost:3000';
const DEFAULT_PASSWORD = process.env.E2E_TEST_USER_PASSWORD ?? '1';

function storageFile(username: string) {
  return path.join(AUTH_DIR, `${username}.json`);
}

async function upsertUser(api: APIRequestContext, username: string) {
  const email = `${username}@example.com`;
  const admin = username === 'admin';
  const res = await api.post(`${API_URL}/api/test/users`, {
    data: { username, email, password: DEFAULT_PASSWORD, admin },
  });
  if (!res.ok()) throw new Error(`upsertUser(${username}) failed ${res.status()}: ${await res.text()}`);
  const json = await res.json();
  if (!json?.success || !json?.data?.id) throw new Error(`upsertUser invalid response: ${JSON.stringify(json)}`);
  return json.data as { id: number; username: string; email: string; admin: boolean };
}

async function login(api: APIRequestContext, username: string) {
  const res = await api.post(`${API_URL}/api/auth/login`, {
    data: { username, password: DEFAULT_PASSWORD },
  });
  if (!res.ok()) throw new Error(`login(${username}) failed ${res.status()}: ${await res.text()}`);
  const json = await res.json();
  const data = json?.data;
  if (!data?.token || !data?.expires_at) throw new Error(`login(${username}) missing token`);
  return { token: data.token as string, expires_at: data.expires_at as string };
}

async function ensureStorageState(
  browser: Browser,
  baseURL: string | undefined,
  api: APIRequestContext,
  username: string,
) {
  const file = storageFile(username);
  if (fs.existsSync(file)) return file;

  fs.mkdirSync(AUTH_DIR, { recursive: true });

  // Create (or update) user + login to get token
  const user = await upsertUser(api, username);
  const { token, expires_at } = await login(api, username);

  // Write a storageState file that seeds localStorage for your app origin
  const origin = new URL(baseURL ?? 'http://localhost:5173').origin;
  const storageState = {
    cookies: [],
    origins: [
      {
        origin,
        localStorage: [
          {
            name: 'auth',
            value: JSON.stringify({ token, expires_at, user: null, modules: [] }),
          },
        ],
      },
    ],
    __meta: { id: user.id, username: user.username },
  };

  fs.writeFileSync(file, JSON.stringify(storageState, null, 2));
  return file;
}

// --- fixtures --------------------------------------------------------------

export const test = base.extend<Fixtures>({
  // API client fixture (points at your API base)
  api: async ({}, use) => {
    const api = await request.newContext({ baseURL: API_URL });
    await use(api);
    await api.dispose();
  },

  // Logged-in page fixture (self-healing storage)
  as: async ({ browser, baseURL, api }, use) => {
    async function openAs(username: string): Promise<Page> {
      const stateFile = await ensureStorageState(browser, baseURL, api, username);
      const ctx = await browser.newContext({ baseURL, storageState: stateFile });
      return ctx.newPage();
    }
    await use(openAs);
  },
});

export const expect = test.expect;
