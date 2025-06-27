import type { PostSubmitAssignmentResponse } from "@/types/modules/assignments/submissions";
import { apiUpload } from "@/utils/api";

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
