import type { 
  PostAssignmentRequest, 
  PostAssignmentResponse, 
  PostUploadAssignmentFileResonse } from "@/types/modules/assignments";
import { api, apiFetch, apiUpload } from "@/utils/api";

export const createAssignment = async (
  moduleId: number,
  payload: PostAssignmentRequest
): Promise<PostAssignmentResponse> => {
  return apiFetch(`/modules/${moduleId}/assignments`, {
    method: "POST",
    body: JSON.stringify(payload),
  });
};

export const uploadAssignmentFile = async (
  moduleId: number,
  assignmentId: number,
  fileType: string,
  file: File
): Promise<PostUploadAssignmentFileResonse> => {
  const form = new FormData();
  form.append("file_type", fileType);
  form.append("file", file);

  return apiUpload(`/modules/${moduleId}/assignments/${assignmentId}/files`, form);
};

export type VerifyAssignmentResponse = {
  /** Optional short tag (useful if you want to show “PIN updated” UI, etc.) */
  password_tag?: string | null;
};

export async function verifyAssignment(
  moduleId: number,
  assignmentId: number,
  pin: string
) {
  return await api.post<VerifyAssignmentResponse>(
    `/modules/${moduleId}/assignments/${assignmentId}/verify`,
    { pin }
  );
}