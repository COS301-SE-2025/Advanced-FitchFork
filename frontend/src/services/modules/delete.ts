import type { DeleteModuleResponse } from "@/types/modules";
import { apiFetch } from "@/utils/api";

export const deleteModule = async (
  moduleId: number
): Promise<DeleteModuleResponse> => {
  return apiFetch(`/modules/${moduleId}`, { method: "DELETE" });
};