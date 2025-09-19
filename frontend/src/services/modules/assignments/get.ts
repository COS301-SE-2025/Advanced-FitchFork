import type { PaginationRequest, PaginationResponse } from "@/types/common";
import type { 
  AssignmentType,
  AssignmentDetails,
  AssignmentReadiness,
  AssignmentFile,
  Assignment,
  AssignmentStats,
} from "@/types/modules/assignments";
import { api, apiDownload, apiFetchBlob, buildQuery } from "@/utils/api";

// ─────────────────────────────────────────────────────────────
// GET Responses Types
// ─────────────────────────────────────────────────────────────

type GetListAssignmentsResponse = {
  assignments: Assignment[];
} & PaginationResponse;

export const listAssignments = async (
  moduleId: number,
  options: {
    name?: string;
    assignment_type?: AssignmentType;
    available_before?: string;
    available_after?: string;
    due_before?: string;
    due_after?: string;
  } & PaginationRequest
) => {
  return api.get<GetListAssignmentsResponse>(`/modules/${moduleId}/assignments?${buildQuery(options)}`);
};

export const getAssignmentDetails = async (
  moduleId: number,
  assignmentId: number
) => {
  return await api.get<AssignmentDetails>(`/modules/${moduleId}/assignments/${assignmentId}`);
};

export const listAssignmentFiles = async (
  moduleId: number,
  assignmentId: number
) => {
  return api.get<AssignmentFile[]>(`/modules/${moduleId}/assignments/${assignmentId}/files`);
};

export const downloadAssignmentFile = async (
  moduleId: number,
  assignmentId: number,
  fileId: number
): Promise<void> => {
  return apiDownload(`/modules/${moduleId}/assignments/${assignmentId}/files/${fileId}`);
};

export const fetchAssignmentFileBlob = async (
  moduleId: number,
  assignmentId: number,
  fileId: number,
): Promise<Blob> => {
  return apiFetchBlob(`/modules/${moduleId}/assignments/${assignmentId}/files/${fileId}`);
};

export const getAssignmentReadiness = async (
  moduleId: number,
  assignmentId: number
) => {
  return api.get<AssignmentReadiness>(`/modules/${moduleId}/assignments/${assignmentId}/readiness`);
};

export async function getAssignmentStats(
  moduleId: number,
  assignmentId: number,
) {
  return await api.get<AssignmentStats>(`/modules/${moduleId}/assignments/${assignmentId}/stats`);
}
