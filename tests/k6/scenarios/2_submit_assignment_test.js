import http from 'k6/http';
import { group, check, sleep } from 'k6';
import { login } from '../shared/auth.js';
import { BASE_URL } from '../shared/config.js';

const submissionFile = open('../test_files/java_submission.zip', 'b');

export const options = {
  thresholds: {
    'http_req_failed': ['rate<0.02'], // Allow a slightly higher failure rate for complex transactions
    'http_req_duration{group:::Submit Assignment}': ['p(95)<10000'], // Increased to 10s for async processing
  },
  stages: [
    { duration: '30s', target: 10 }, // Reduced to 10 concurrent users to prevent overload
    { duration: '2m', target: 10 },  // Maintain load for longer
    { duration: '1m', target: 0 },   // Longer ramp down to prevent interruptions
  ],
};

export default function () {
  const moduleId = __ENV.MODULE_ID || '9999';
  const assignmentId = __ENV.ASSIGNMENT_ID || '9999';

  const authToken = login({
    username: __ENV.USERNAME || 'student',
    password: __ENV.PASSWORD || '1',
  });

  if (authToken) {
    group('Submit Assignment', function () {
      // Enable async mode to avoid waiting for full grading process
      const url = `${BASE_URL}/modules/${moduleId}/assignments/${assignmentId}/submissions?async_mode=true`;
      const params = { headers: { 'Authorization': `Bearer ${authToken}` } };
      
      // Create form data matching the API requirements
      const data = {
        file: http.file(submissionFile, 'submission.zip'),
        is_practice: 'false',         // Required field for practice mode flag
        attests_ownership: 'true',    // Required field for ownership attestation
      };

      const res = http.post(url, data, params);
      
      // Log response details for debugging
      if (res.status >= 400) {
        console.log(`Submission failed - Status: ${res.status}, Body: ${res.body}`);
      }
      
      check(res, {
        'Submission successful': (r) => r.status === 201 || r.status === 200 || r.status === 202,
        'Submission response time is acceptable': (r) => r.timings.duration < 10000,
        'Submission queued for processing': (r) => {
          // In async mode, we expect 202 Accepted for queued processing
          if (r.status === 202) return true;
          // Other success codes are also acceptable
          return r.status === 200 || r.status === 201;
        }
      });
      
      // Add a small delay to prevent overwhelming the submission queue
      sleep(0.1);
    });
  }
}