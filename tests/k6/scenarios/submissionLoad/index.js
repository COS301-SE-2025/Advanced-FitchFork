// 📁 k6/scenarios/fullSubmissionFlow/index.js
import { login } from '../../shared/auth.js';
import { createModule } from './phases/01_createModule.js';
import { setupAssignment } from './phases/02_setupAssignment.js';
import { registerUsers } from './phases/03_registerUsers.js';
import { enrollStudents } from './phases/04_enrollUsers.js';
import { uploadSubmission } from './phases/05_uploadSubmission.js';

export const options = {
  vus: 15,
  iterations: 15,
};

export function setup() {
  const NUM_USERS = 15;
  const adminToken = login({ username: 'admin', password: '1' });

  const moduleId = createModule(adminToken);
  const assignmentId = setupAssignment(moduleId, adminToken);

  const { usernames, userIds } = registerUsers(NUM_USERS);
  enrollStudents(userIds, moduleId, adminToken);

  return {
    moduleId,
    assignmentId,
    usernames,
  };
}

export default function (data) {
  uploadSubmission(data);
}
