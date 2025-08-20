import type { AssignmentConfig } from "@/types/modules/assignments/config";
import { api } from "@/utils/api";


export async function setAssignmentConfig(
  moduleId: number,
  assignmentId: number,
  config: AssignmentConfig
) {
  return await api.post<AssignmentConfig>(`/modules/${moduleId}/assignments/${assignmentId}/config`, config);
}

export async function resetAssignmentConfig(
  moduleId: number,
  assignmentId: number
) {
  return await api.post<AssignmentConfig>(`/modules/${moduleId}/assignments/${assignmentId}/config/reset`);
}