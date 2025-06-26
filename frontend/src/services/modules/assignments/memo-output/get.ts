
import type { GetMemoOutputResponse } from "@/types/modules/assignments/memo-output/responses";
import { apiFetch } from "@/utils/api";

export const getMemoOutput = async (
  moduleId: number,
  assignmentId: number
): Promise<GetMemoOutputResponse> => {
  return apiFetch(`/modules/${moduleId}/assignments/${assignmentId}/memo-output`);
};
