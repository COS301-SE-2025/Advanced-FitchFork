import type { ApiResponse } from "@/types/common";
import { api } from "@/utils/api";


export async function deleteInterpreter(
  moduleId: number,
  assignmentId: number
): Promise<ApiResponse<{}>> {
  return api.del<{}>(`/modules/${moduleId}/assignments/${assignmentId}/interpreter`);
}