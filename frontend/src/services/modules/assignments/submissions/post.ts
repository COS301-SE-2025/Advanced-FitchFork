import type { ApiResponse } from "@/types/common";
import type { PostSubmitAssignmentResponse, RemarkResponse } from "@/types/modules/assignments/submissions";
import type { RemarkRequest } from "@/types/modules/assignments/submissions/requests";
import { apiFetch, apiUpload } from "@/utils/api";

export const submitAssignment = async (
  moduleId: number,
  assignmentId: number,
  file: File,
  isPractice?: boolean
): Promise<PostSubmitAssignmentResponse> => {
  const formData = new FormData();
  formData.append("file", file);
  if (isPractice !== undefined) {
    formData.append("is_practice", isPractice ? "true" : "false");
  }

  return apiUpload(`/modules/${moduleId}/assignments/${assignmentId}/submissions`, formData);
};

export const remarkSubmissions = async (
  moduleId: number,
  assignmentId: number,
  payload: RemarkRequest
): Promise<ApiResponse<RemarkResponse>> => {
  return apiFetch(`/modules/${moduleId}/assignments/${assignmentId}/submissions/remark`, { 
      method: "POST",
      body: JSON.stringify(payload)
    });
};