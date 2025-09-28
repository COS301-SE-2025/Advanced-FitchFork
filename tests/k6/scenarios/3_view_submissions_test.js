import http from 'k6/http';
import { group, check, sleep } from 'k6';
import { login } from '../shared/auth.js';
import { BASE_URL } from '../shared/config.js';

export const options = {
  thresholds: {
    'http_req_duration': ['p(95)<2000'],
  },
  vus: 10,
  duration: '1m',
};

export default function () {
  const authToken = login({
    username: __ENV.LECTURER_USERNAME || 'lecturer',
    password: __ENV.LECTURER_PASSWORD || '1',
  });

  const moduleId = __ENV.MODULE_ID || '9999';
  const assignmentId = __ENV.ASSIGNMENT_ID_LARGE || '9999';

  if (authToken) {
    group('View All Submissions', function () {
      const params = { headers: { 'Authorization': `Bearer ${authToken}` } };
      const res = http.get(`${BASE_URL}/modules/${moduleId}/assignments/${assignmentId}/submissions`, params);
      check(res, {
        'Successfully fetched submissions': (r) => r.status === 200,
        'Response contains a submissions array': (r) => r.json('data.submissions') !== undefined,
      });
    });
  }
  sleep(3);
}