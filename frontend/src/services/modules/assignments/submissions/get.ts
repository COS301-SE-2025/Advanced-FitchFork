import type { PaginationRequest } from "@/types/common";
import type { GetSubmissionListResponse, GetSubmissionDetailResponse } from "@/types/modules/assignments/submissions";
import { apiFetch, buildQuery } from "@/utils/api";


export const getSubmissions = async (
  moduleId: number,
  assignmentId: number,
  options: {
    username?: string;
    status?: string;
  } & PaginationRequest
): Promise<GetSubmissionListResponse> => {
  const query = buildQuery(options);
  return apiFetch(`/modules/${moduleId}/assignments/${assignmentId}/submissions?${query}`);
};

export const getSubmissionDetails = async (
  moduleId: number,
  assignmentId: number,
  submissionId: number
): Promise<GetSubmissionDetailResponse> => {
  return apiFetch(`/modules/${moduleId}/assignments/${assignmentId}/submissions/${submissionId}`);
};
