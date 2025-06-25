import type { AssignmentConfig, PostAssignmentConfigResponse } from "@/types/modules/assignments/config";
import { apiFetch } from "@/utils/api";


export async function setAssignmentConfig(
  moduleId: number,
  assignmentId: number,
  config: AssignmentConfig
) {
  return await apiFetch<PostAssignmentConfigResponse>(
    `/modules/${moduleId}/assignments/${assignmentId}/config`,
    {
      method: 'POST',
      body: JSON.stringify(config),
    }
  );
}