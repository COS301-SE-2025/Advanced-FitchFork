import type { ApiResponse } from "@/types/common";
import { apiFetch } from "@/utils/api";

export const generateMemoOutput = async (
moduleId: number,
assignmentId: number
): Promise<ApiResponse<null>> => {
  return apiFetch(`/modules/${moduleId}/assignments/${assignmentId}/memo_output/generate`,
    { method: "POST" }
  );
}