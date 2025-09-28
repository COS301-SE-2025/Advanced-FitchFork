import http from 'k6/http';
import { group, check, sleep } from 'k6';
import { login } from '../shared/auth.js';
import { BASE_URL } from '../shared/config.js';
import { randomString } from 'https://jslib.k6.io/k6-utils/1.2.0/index.js';

export const options = {
  thresholds: {
    'http_req_duration': ['p(95)<600'],
  },
  vus: 5,
  iterations: 20,
};

export default function () {
  const authToken = login({
    username: __ENV.LECTURER_USERNAME || 'lecturer',
    password: __ENV.LECTURER_PASSWORD || '1',
  });
  const moduleId = __ENV.MODULE_ID || '1';

  if (authToken) {
    group('Create Announcement', function () {
      const url = `${BASE_URL}/modules/${moduleId}/announcements`;
      const params = {
        headers: {
          'Authorization': `Bearer ${authToken}`,
          'Content-Type': 'application/json',
        },
      };
      const payload = JSON.stringify({
        title: `Performance Test Announcement ${randomString(5)}`,
        body: 'This is an automated message from a k6 performance test.',
        pinned: false,
      });

      const res = http.post(url, payload, params);
      
      if (res.status !== 200 && res.status !== 201) {
        console.log(`Announcement creation failed - Status: ${res.status}, Body: ${res.body}`);
      }
      
      check(res, { 
        'Announcement created successfully': (r) => r.status === 201 || r.status === 200,
        'Response time acceptable': (r) => r.timings.duration < 600,
      });
    });
  }
}