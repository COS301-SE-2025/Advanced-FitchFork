import fs from 'node:fs';
import path from 'node:path';
import { request } from '@playwright/test';
import { AUTH_DIR } from '@helpers/constants';

export default async function globalTeardown() {
  const apiURL = process.env.E2E_API_URL ?? 'http://localhost:3000';

  if (fs.existsSync(AUTH_DIR)) {
    const api = await request.newContext({ baseURL: apiURL });

    for (const file of fs.readdirSync(AUTH_DIR)) {
      if (!file.endsWith('.json')) continue;

      const data = JSON.parse(fs.readFileSync(path.join(AUTH_DIR, file), 'utf8'));
      const id = data?.__meta?.id;
      const username = data?.__meta?.username;

      if (id) {
        try {
          const res = await api.delete(`/api/test/users/${id}`);
          if (res.ok()) {
            console.log(`Deleted test user '${username}' (id: ${id})`);
          } else {
            console.warn(`Could not delete test user '${username}' (status ${res.status()})`);
          }
        } catch (err) {
          console.error(`Error deleting test user '${username}':`, err);
        }
      }
    }

    await api.dispose();

    fs.rmSync(AUTH_DIR, { recursive: true, force: true });
    console.log(`Removed auth storage directory: ${AUTH_DIR}`);
  }
}
