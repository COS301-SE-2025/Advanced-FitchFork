import { apiFetch } from "@/utils/api";
import type { AssignmentConfig } from "@/types/modules/assignments/config";

export async function getAssignmentConfig(moduleId: number, assignmentId: number) {
  return await apiFetch<AssignmentConfig>(
    `/modules/${moduleId}/assignments/${assignmentId}/config`,
    { method: 'GET' }
  );
}