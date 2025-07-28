import type { 
  PostAssignPersonnelRequest, 
  PostAssignPersonnelResponse 
} from "@/types/modules/personnel";
import { apiFetch } from "@/utils/api";

export const assignPersonnel = async (
  moduleId: number,
  payload: PostAssignPersonnelRequest
): Promise<PostAssignPersonnelResponse> => {
  return apiFetch(`/modules/${moduleId}/personnel`, {
    method: "POST",
    body: JSON.stringify(payload),
  });
};
