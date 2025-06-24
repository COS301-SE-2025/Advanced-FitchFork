import type { PutAssignmentRequest, PutAssignmentResponse } from "@/types/modules/assignments";
import { apiFetch } from "@/utils/api";

export const editAssignment = async (
  moduleId: number,
  assignmentId: number,
  payload: PutAssignmentRequest
): Promise<PutAssignmentResponse> => {
  return apiFetch(`/modules/${moduleId}/assignments/${assignmentId}`, {
    method: "PUT",
    body: JSON.stringify(payload),
  });
};
