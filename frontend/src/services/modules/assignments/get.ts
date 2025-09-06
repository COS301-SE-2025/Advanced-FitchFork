import type { PaginationRequest } from "@/types/common";
import type { 
  GetListAssignmentsResponse, 
  GetAssignmentResponse, 
  Assignment, 
  AssignmentFile, 
  GetListAssignmentFilesResponse, 
  AssignmentType,
  GetAssignmentReadinessResponse,
  BestMark} from "@/types/modules/assignments";
import { apiDownload, apiFetch, buildQuery } from "@/utils/api";

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
): Promise<GetListAssignmentsResponse> => {
  return apiFetch(`/modules/${moduleId}/assignments?${buildQuery(options)}`);
};

export const getAssignmentDetails = async (
  moduleId: number,
  assignmentId: number
): Promise<GetAssignmentResponse> => {
  const res = await apiFetch<{
    assignment: Assignment;
    files: AssignmentFile[];
    best_mark?: BestMark;
  }>(`/modules/${moduleId}/assignments/${assignmentId}`);

  const merged = {
    ...res.data.assignment,
    files: res.data.files,
    best_mark: res.data.best_mark,
  };

  return {
    success: res.success,
    message: res.message,
    data: merged,
  };
};

export const listAssignmentFiles = async (
  moduleId: number,
  assignmentId: number
): Promise<GetListAssignmentFilesResponse> => {
  return apiFetch(`/modules/${moduleId}/assignments/${assignmentId}/files`);
};

export const downloadAssignmentFile = async (
  moduleId: number,
  assignmentId: number,
  fileId: number
): Promise<void> => {
  return apiDownload(`/modules/${moduleId}/assignments/${assignmentId}/files/${fileId}`);
};

export const getAssignmentReadiness = async (
  moduleId: number,
  assignmentId: number
): Promise<GetAssignmentReadinessResponse> => {
  return apiFetch(`/modules/${moduleId}/assignments/${assignmentId}/readiness`);
};