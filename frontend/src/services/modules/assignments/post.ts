import type { 
  PostAssignmentRequest, 
  PostAssignmentResponse, 
  PostUploadAssignmentFilesResonse } from "@/types/modules/assignments";
import { apiFetch, apiUpload } from "@/utils/api";

export const createAssignment = async (
  moduleId: number,
  payload: PostAssignmentRequest
): Promise<PostAssignmentResponse> => {
  return apiFetch(`/modules/${moduleId}/assignments`, {
    method: "POST",
    body: JSON.stringify(payload),
  });
};

export const uploadAssignmentFiles = async (
  moduleId: number,
  assignmentId: number,
  files: File[]
): Promise<PostUploadAssignmentFilesResonse> => {
  const form = new FormData();
  for (const file of files) {
    form.append("files[]", file);
  }

  return apiUpload(`/modules/${moduleId}/assignments/${assignmentId}/files`, form);
};