import { apiFetch } from "@/utils/api";
import type { GetExecutionConfigResponse } from "@/types/assignments/config";

export const getAssignmentConfig = async (
  moduleId: number,
  assignmentId: number
): Promise<GetExecutionConfigResponse> => {
  return apiFetch(`/modules/${moduleId}/assignments/${assignmentId}/config`);
};
