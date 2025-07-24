import type { ApiResponse } from "@/types/common";
import type {
  BulkUpdateAssignmentsRequest,
  BulkUpdateAssignmentsResponse,
  PutAssignmentRequest,
  PutAssignmentResponse,
} from "@/types/modules/assignments";
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

export const bulkUpdateAssignments = async (
  moduleId: number,
  payload: BulkUpdateAssignmentsRequest
): Promise<BulkUpdateAssignmentsResponse> => {
  return apiFetch(`/modules/${moduleId}/assignments/bulk`, {
    method: "PUT",
    body: JSON.stringify(payload),
  });
};

export const openAssignment = async (
  moduleId: number,
  assignmentId: number
): Promise<ApiResponse<null>> => {
  return apiFetch(`/modules/${moduleId}/assignments/${assignmentId}/open`, {
    method: "PUT",
  });
};

export const closeAssignment = async (
  moduleId: number,
  assignmentId: number
): Promise<ApiResponse<null>> => {
  return apiFetch(`/modules/${moduleId}/assignments/${assignmentId}/close`, {
    method: "PUT",
  });
};
