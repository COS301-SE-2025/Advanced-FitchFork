import type { 
  DeletePersonnelRequest,
  DeletePersonnelResponse 
} from "@/types/modules/personnel";
import { apiFetch } from "@/utils/api";

export const removePersonnel = async (
  moduleId: number,
  payload: DeletePersonnelRequest
): Promise<DeletePersonnelResponse> => {
  return apiFetch(`/modules/${moduleId}/personnel`, {
    method: "DELETE",
    body: JSON.stringify(payload),
  });
};
