import type { GetMemoOutputResponse } from "@/types/assignments/memo-output";
import { apiFetch } from "@/utils/api";

export const getMemoOutput = async (
  moduleId: number,
  assignmentId: number
): Promise<GetMemoOutputResponse> => {
  return apiFetch(`/modules/${moduleId}/assignments/${assignmentId}/memo-output`);
};
