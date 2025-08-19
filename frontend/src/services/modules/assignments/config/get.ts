import { api } from "@/utils/api";
import type { AssignmentConfig } from "@/types/modules/assignments/config";

export async function getAssignmentConfig(moduleId: number, assignmentId: number) {
  return await api.get<AssignmentConfig>(`/modules/${moduleId}/assignments/${assignmentId}/config`);
}