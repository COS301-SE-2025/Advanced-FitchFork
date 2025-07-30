// 📁 k6/scenarios/fullSubmissionFlow/phases/04_enrollUsers.js
import { authorizedPost } from '../../../shared/http.js';
import { BASE_URL } from '../../../shared/config.js';

/**
 * Enroll registered users into a module
 */
export function enrollStudents(userIds, moduleId, adminToken) {
  const assignRes = authorizedPost(
    `${BASE_URL}/modules/${moduleId}/personnel`,
    { user_ids: userIds, role:"student" },
    adminToken
  );

  if (assignRes.status !== 200) {
    console.error(`❌ Failed to assign students. Response: ${assignRes.body}`);
  } else {
    console.log(`✅ Assigned ${userIds.length} users to module ${moduleId}`);
  }
}
