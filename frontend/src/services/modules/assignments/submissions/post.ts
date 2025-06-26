import type { ApiResponse } from "@/types/common";
import { apiUpload } from "@/utils/api";

export const submitAssignment = async (
  moduleId: number,
  assignmentId: number
): Promise<ApiResponse<null>> => {
  return apiUpload(`/modules/${moduleId}/assignments/${assignmentId}/submissions`, new FormData);
}