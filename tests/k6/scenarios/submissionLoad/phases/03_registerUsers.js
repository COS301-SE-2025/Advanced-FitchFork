// 📁 k6/scenarios/fullSubmissionFlow/phases/03_registerUsers.js
import http from 'k6/http';
import { BASE_URL } from '../../../shared/config.js';

/**
 * Register `count` users and return an array of their usernames and IDs
 */
export function registerUsers(count) {
  const requests = [];
  const usernames = [];

  for (let i = 0; i < count; i++) {
    const username = `u${10000000 + i}`;
    const email = `${username}@test.local`;
    const payload = JSON.stringify({
      username,
      email,
      password: 'password123',
    });

    usernames.push(username);

    requests.push({
      method: 'POST',
      url: `${BASE_URL}/auth/register`,
      body: payload,
      params: { headers: { 'Content-Type': 'application/json' } },
    });
  }

  const responses = http.batch(requests);
  const userIds = [];

  for (let i = 0; i < responses.length; i++) {
    const res = responses[i];
    const json = res.json?.();

    if (res.status === 201) {
      const id = json?.data?.id;
      if (!id) {
        console.error(`❌ No ID returned for ${usernames[i]}`);
        continue;
      }
      userIds.push(id);
    } else if (res.status === 409) {
      console.warn(`⚠️  User ${usernames[i]} already exists, skipping`);
    } else {
      console.error(`❌ Failed to register ${usernames[i]}: ${json?.message || res.body}`);
    }
  }

  return { usernames, userIds };
}
