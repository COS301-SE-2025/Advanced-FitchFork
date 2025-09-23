import type { ApiResponse } from "@/types/common";
import type { PostSubmitAssignmentResponse, RemarkResponse, ResubmitRequest, ResubmitResponse } from "@/types/modules/assignments/submissions";
import type { RemarkRequest } from "@/types/modules/assignments/submissions/requests";
import { api, apiUpload } from "@/utils/api";

export const submitAssignment = async (
  moduleId: number,
  assignmentId: number,
  file: File,
  isPractice: boolean,
  attestsOwnership: boolean,
  asyncMode: boolean = false 
): Promise<PostSubmitAssignmentResponse> => {
  const formData = new FormData();
  formData.append("file", file);
  formData.append("is_practice", String(isPractice));
  formData.append("attests_ownership", String(attestsOwnership));

  const qs = new URLSearchParams({ async_mode: String(asyncMode) });

  return apiUpload(
    `/modules/${moduleId}/assignments/${assignmentId}/submissions?${qs.toString()}`,
    formData
  );
};


export const remarkSubmissions = async (
  moduleId: number,
  assignmentId: number,
  payload: RemarkRequest
): Promise<ApiResponse<RemarkResponse>> => {
  return api.post(`/modules/${moduleId}/assignments/${assignmentId}/submissions/remark`, payload);
};

// Service function
export const resubmitSubmissions = async (
  moduleId: number,
  assignmentId: number,
  payload: ResubmitRequest
): Promise<ApiResponse<ResubmitResponse>> => {
  return api.post(`/modules/${moduleId}/assignments/${assignmentId}/submissions/resubmit`, payload);
};
