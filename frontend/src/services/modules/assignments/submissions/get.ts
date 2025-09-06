import type { ApiResponse, PaginationRequest } from "@/types/common";
import type { GetSubmissionListResponse, GetSubmissionDetailResponse, SubmissionTaskOutput } from "@/types/modules/assignments/submissions";
import { apiDownload, apiFetch, apiFetchBlob, buildQuery } from "@/utils/api";

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

export async function getSubmissionOutput(
  moduleId: number,
  assignmentId: number,
  submissionId: number
): Promise<ApiResponse<SubmissionTaskOutput[]>> {
  return apiFetch(
    `/modules/${moduleId}/assignments/${assignmentId}/submissions/${submissionId}/output`
  );
}

// For opening in IDE (needs Blob)
export const downloadSubmissionFile = async (
  moduleId: number,
  assignmentId: number,
  submissionId: number
): Promise<Blob> => {
  return apiFetchBlob(`/modules/${moduleId}/assignments/${assignmentId}/submissions/${submissionId}/download`);
};

// (Optional) If you also need a click-to-save button elsewhere, keep using:
export const downloadSubmissionFileToDisk = async (
  moduleId: number,
  assignmentId: number,
  submissionId: number
): Promise<void> => {
  return apiDownload(`/modules/${moduleId}/assignments/${assignmentId}/submissions/${submissionId}/download`);
};
