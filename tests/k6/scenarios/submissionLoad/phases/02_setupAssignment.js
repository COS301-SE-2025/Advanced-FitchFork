import { authorizedPost, authorizedFileUpload } from '../../../shared/http.js';
import { BASE_URL } from '../../../shared/config.js';
import { check } from 'k6';
import http from 'k6/http';

// Load files once
const configFile = open('../../../test_files/config.json', 'b');
const mainFile = open('../../../test_files/java_main.zip', 'b');
const memoFile = open('../../../test_files/java_memo.zip', 'b');
const makefile = open('../../../test_files/java_makefile.zip', 'b');

/**
 * Create an assignment and upload everything it needs
 */
export function setupAssignment(moduleId, adminToken) {
  // 1. Create assignment
  const now = new Date();
  const available_from = now.toISOString();
  const due_date = new Date(now.getTime() + 7 * 86400000).toISOString(); // 7 days later

  const assignmentRes = authorizedPost(
    `${BASE_URL}/modules/${moduleId}/assignments`,
    {
      name: 'Autograded Assignment',
      description: 'Generated during k6 setup',
      assignment_type: 'assignment',
      available_from,
      due_date,
    },
    adminToken
  );

  check(assignmentRes, { 'assignment created': (r) => r.status === 201 });
  const assignmentId = assignmentRes.json('data.id');

  // 2. Upload config + other files
  uploadFile(moduleId, assignmentId, 'config', configFile, 'config.json', adminToken);
  uploadFile(moduleId, assignmentId, 'main', mainFile, 'java_main.zip', adminToken);
  uploadFile(moduleId, assignmentId, 'memo', memoFile, 'java_memo.zip', adminToken);
  uploadFile(moduleId, assignmentId, 'makefile', makefile, 'java_makefile.zip', adminToken);

  // 3. Create specific tasks with defined names and commands
  createTask(moduleId, assignmentId, 1, 'make task1', adminToken);
  createTask(moduleId, assignmentId, 2, 'make task2', adminToken);
  createTask(moduleId, assignmentId, 3, 'make task3', adminToken);

  // 4. Generate memo outputs
  const memoRes = authorizedPost(
    `${BASE_URL}/modules/${moduleId}/assignments/${assignmentId}/memo_output/generate`,
    {},
    adminToken
  );
  check(memoRes, { 'memo generated': (r) => r.status === 200 });

  // 5. Generate mark allocator
  const markRes = authorizedPost(
    `${BASE_URL}/modules/${moduleId}/assignments/${assignmentId}/mark_allocator/generate`,
    {},
    adminToken
  );
  check(markRes, { 'mark allocator generated': (r) => r.status === 200 });

  return assignmentId;
}

function uploadFile(moduleId, assignmentId, type, content, filename, token) {
  const res = authorizedFileUpload(
    `${BASE_URL}/modules/${moduleId}/assignments/${assignmentId}/files`,
    {
      file_type: type,
      file: http.file(content, filename, 'application/octet-stream'),
    },
    token
  );
  check(res, { [`${type} uploaded`]: (r) => r.status === 201 });
}

function createTask(moduleId, assignmentId, taskNumber, command, token) {
  const res = authorizedPost(
    `${BASE_URL}/modules/${moduleId}/assignments/${assignmentId}/tasks`,
    { name: `Task ${taskNumber}`,task_number: taskNumber, command },
    token
  );
  check(res, { [`task ${taskNumber} created`]: (r) => r.status === 201 });
}
