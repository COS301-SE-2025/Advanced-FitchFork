import type { GetSubmissionListResponse, GetSubmissionDetailResponse } from "@/types/modules/assignments/submissions";
import { apiFetch } from "@/utils/api";


export const getSubmissions = async (
  moduleId: number,
  assignmentId: number,
  query: URLSearchParams
): Promise<GetSubmissionListResponse> => {
  return apiFetch(`/modules/${moduleId}/assignments/${assignmentId}/submissions?${query.toString()}`);
};

export const getSubmissionDetails = async (
  moduleId: number,
  assignmentId: number,
  submissionId: number
): Promise<GetSubmissionDetailResponse> => {
  return apiFetch(`/modules/${moduleId}/assignments/${assignmentId}/submissions/${submissionId}`);
};
