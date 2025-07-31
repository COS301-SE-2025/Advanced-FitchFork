import { BASE_URL } from '../../../shared/config.js';
import { login } from '../../../shared/auth.js';
import http from 'k6/http';
import { check } from 'k6';

const SUBMISSION_ZIP = open('../../../test_files/java_submission.zip', 'b');

export function uploadSubmission(data) {
  const { moduleId, assignmentId, userIds } = data;
  const vuIndex = __VU - 1;

  const username = `u${10000000 + vuIndex}`;
  const token = login({ username, password: 'password123' });

  const res = http.post(
    `${BASE_URL}/modules/${moduleId}/assignments/${assignmentId}/submissions`,
    {
      file: http.file(SUBMISSION_ZIP, 'java_submission.zip', 'application/zip'),
      is_practice: 'false',
    },
    {
      headers: { Authorization: `Bearer ${token}` },
    }
  );

  check(res, {
    [`user ${vuIndex} submitted`]: (r) => r.status === 200 || r.status === 201,
  });
}
