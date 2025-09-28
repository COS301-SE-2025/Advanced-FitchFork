import type { PaginationRequest, PaginationResponse } from "@/types/common";
import type { SubmissionTaskOutput, Submission } from "@/types/modules/assignments/submissions";
import { api, apiDownload, apiFetchBlob, buildQuery } from "@/utils/api";


export type GetSubmissionListResponse = {
  submissions: Submission[];
} & PaginationResponse;

export const getSubmissions = async (
  moduleId: number,
  assignmentId: number,
  options: {
    username?: string;
    status?: string;
  } & PaginationRequest
) => {
  const query = buildQuery(options);
  return api.get<GetSubmissionListResponse>(`/modules/${moduleId}/assignments/${assignmentId}/submissions?${query}`);
};

export const getSubmissionDetails = async (
  moduleId: number,
  assignmentId: number,
  submissionId: number
) => {
  return api.get<Submission>(`/modules/${moduleId}/assignments/${assignmentId}/submissions/${submissionId}`);
};

export async function getSubmissionOutput(
  moduleId: number,
  assignmentId: number,
  submissionId: number
) {
  return api.get<SubmissionTaskOutput[]>(
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