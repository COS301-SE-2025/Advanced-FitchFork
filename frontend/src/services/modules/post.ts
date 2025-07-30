import type { 
  PostModuleRequest, 
  PostModuleResponse} from "@/types/modules";
import { apiFetch } from "@/utils/api";

export const createModule = async (
  payload: PostModuleRequest
): Promise<PostModuleResponse> => {
  return apiFetch(`/modules`, {
    method: "POST",
    body: JSON.stringify(payload),
  });
};