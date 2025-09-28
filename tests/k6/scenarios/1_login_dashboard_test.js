import http from 'k6/http';
import { group, check } from 'k6';
import { login } from '../shared/auth.js';
import { BASE_URL } from '../shared/config.js';

export const options = {
  thresholds: {
    'http_req_failed': ['rate<0.01'],
    'http_req_duration': ['p(95)<3000'],
    'group_duration{group:::Login}': ['p(95)<3000'],
    'group_duration{group:::Load Dashboard}': ['p(95)<3000'],
  },
  stages: [
    { duration: '30s', target: 50 }, // Ramp up to 50 users
    { duration: '1m', target: 50 },  // Stay at 50 users
    { duration: '10s', target: 0 },   // Ramp down
  ],
};

export default function () {
  let authToken;

  group('Login', function () {
    authToken = login({
      username: __ENV.USERNAME || 'student',
      password: __ENV.PASSWORD || '1',
    });
    check(authToken, { 'Authenticated successfully': (token) => token != null });
  });

  if (authToken) {
    group('Load Dashboard', function () {
      const params = { headers: { 'Authorization': `Bearer ${authToken}` } };
      
      const announcementsRes = http.get(`${BASE_URL}/me/announcements`, params);
      const assignmentsRes = http.get(`${BASE_URL}/me/assignments`, params);
      
      check(announcementsRes, { 'Announcements loaded successfully': (r) => r.status === 200 });
      check(assignmentsRes, { 'Assignments loaded successfully': (r) => r.status === 200 });
    });
  }
}