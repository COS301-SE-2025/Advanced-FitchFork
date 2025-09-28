import type { ApiResponse } from "@/types/common";
import type { CreateStarterRequest } from "@/types/modules/assignments/starter";
import { api } from "@/utils/api";

export function createStarter(
  moduleId: number,
  assignmentId: number,
  payload: CreateStarterRequest
): Promise<ApiResponse<unknown>> {
  return api.post<unknown>(
    `/modules/${moduleId}/assignments/${assignmentId}/starter`,
    payload
  );
}
