import type { AssignmentConfig, UpdateAssignmentConfigResponse } from "@/types/modules/assignments/config";
import { apiFetch } from "@/utils/api";

export async function updateAssignmentConfig(
  moduleId: number,
  assignmentId: number,
  update: Partial<AssignmentConfig>
) {
  return await apiFetch<UpdateAssignmentConfigResponse>(
    `/modules/${moduleId}/assignments/${assignmentId}/config`,
    {
      method: 'PUT',
      body: JSON.stringify(update),
    }
  );
}