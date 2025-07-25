import type { DeleteAssignmentResponse, DeleteAssignmentFilesResponse, BulkDeleteAssignmentsRequest, BulkDeleteAssignmentsResponse } from "@/types/modules/assignments";
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

export const bulkDeleteAssignments = async (
  moduleId: number,
  payload: BulkDeleteAssignmentsRequest
): Promise<BulkDeleteAssignmentsResponse> => {
  return apiFetch(`/modules/${moduleId}/assignments/bulk`, {
    method: "DELETE",
    body: JSON.stringify(payload),
  });
};