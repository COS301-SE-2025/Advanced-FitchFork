import type { DeleteAssignmentResponse, DeleteAssignmentFilesResponse } from "@/types/modules/assignments";
import { apiFetch } from "@/utils/api";

export const deleteAssignment = async (
  moduleId: number,
  assignmentId: number
): Promise<DeleteAssignmentResponse> => {
  return apiFetch(`/modules/${moduleId}/assignments/${assignmentId}`, {
    method: "DELETE",
  });
};

export const deleteAssignmentFiles = async (
  moduleId: number,
  assignmentId: number,
  payload: { file_ids: number[] }
): Promise<DeleteAssignmentFilesResponse> => {
  return apiFetch(`/modules/${moduleId}/assignments/${assignmentId}/files`, {
    method: "DELETE",
    body: JSON.stringify(payload),
  });
};