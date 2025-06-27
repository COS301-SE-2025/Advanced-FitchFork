import { apiFetch } from "@/utils/api";
import type { GetAssignmentConfigResponse } from "@/types/modules/assignments/config";

export async function getAssignmentConfig(moduleId: number, assignmentId: number) {
  return await apiFetch<GetAssignmentConfigResponse>(
    `/modules/${moduleId}/assignments/${assignmentId}/config`,
    { method: 'GET' }
  );
}