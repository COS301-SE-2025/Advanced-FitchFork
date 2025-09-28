import http from 'k6/http';
import { group, check, sleep } from 'k6';
import { login } from '../shared/auth.js';
import { BASE_URL } from '../shared/config.js';

export const options = {
  thresholds: {
    'http_req_failed': ['rate<0.01'],
    'http_req_duration': ['p(95)<3000'],
    'group_duration{group:::Assignment Statistics}': ['p(95)<4000'],
    'group_duration{group:::Submission Analysis}': ['p(95)<5000'],
  },
  stages: [
    { duration: '30s', target: 10 }, // Ramp up to 10 concurrent users
    { duration: '2m', target: 10 },  // Hold the load
    { duration: '30s', target: 0 },  // Ramp down
  ],
};

export default function () {
  const authToken = login({
    username: __ENV.LECTURER_USERNAME || 'lecturer',
    password: __ENV.LECTURER_PASSWORD || '1',
  });

  const moduleId = __ENV.MODULE_ID || '9999';
  const assignmentId = __ENV.ASSIGNMENT_ID || '9999';

  if (authToken) {
    const params = { headers: { 'Authorization': `Bearer ${authToken}` } };

    group('Assignment Statistics', function () {
      const statsRes = http.get(`${BASE_URL}/modules/${moduleId}/assignments/${assignmentId}/stats`, params);
      check(statsRes, {
        'Submission stats computed': (r) => r.status === 200,
        'Statistics contain data': (r) => {
          if (r.status === 200) {
            const data = r.json('data');
            return data !== null && data !== undefined;
          }
          return false;
        },
      });

      const configRes = http.get(`${BASE_URL}/modules/${moduleId}/assignments/${assignmentId}/config`, params);
      check(configRes, {
        'Assignment config loaded': (r) => r.status === 200,
        'Config has policy data': (r) => {
          if (r.status === 200) {
            const data = r.json('data');
            return data && (data.policy || data.execution);
          }
          return false;
        },
      });

      sleep(0.1);
    });

    group('Submission Analysis', function () {
      const submissionsRes = http.get(`${BASE_URL}/modules/${moduleId}/assignments/${assignmentId}/submissions?per_page=50`, params);
      check(submissionsRes, {
        'Submissions list loaded': (r) => r.status === 200,
        'Submissions data present': (r) => {
          if (r.status === 200) {
            const data = r.json('data');
            return data && data.submissions !== undefined;
          }
          return false;
        },
      });

      if (submissionsRes.status === 200) {
        const submissionsData = submissionsRes.json('data');
        if (submissionsData && submissionsData.submissions && submissionsData.submissions.length > 0) {
          const firstSubmission = submissionsData.submissions[0];

          const submissionRes = http.get(
            `${BASE_URL}/modules/${moduleId}/assignments/${assignmentId}/submissions/${firstSubmission.id}`, 
            params
          );
          check(submissionRes, {
            'Submission details loaded': (r) => r.status === 200,
            'Submission has marks data': (r) => {
              if (r.status === 200) {
                const data = r.json('data');
                return data && (data.mark !== undefined || data.tasks !== undefined);
              }
              return false;
            },
          });

          const outputRes = http.get(
            `${BASE_URL}/modules/${moduleId}/assignments/${assignmentId}/submissions/${firstSubmission.id}/output`, 
            params
          );
          check(outputRes, {
            'Submission output accessible': (r) => r.status === 200 || r.status === 403,
          });
        }
      }

      sleep(0.2);
    });

    sleep(1);
  }
}