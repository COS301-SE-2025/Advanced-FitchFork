import type { PutModuleRequest, PutModuleResponse } from "@/types/modules";
import { apiFetch } from "@/utils/api";

export const editModule = async (
  moduleId: number,
  data: PutModuleRequest
): Promise<PutModuleResponse> => {
  return apiFetch(`/modules/${moduleId}`, {
    method: "PUT",
    body: JSON.stringify(data),
  });
};