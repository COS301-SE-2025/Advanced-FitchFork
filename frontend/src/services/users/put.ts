import type { PutUserReponse, PutUserRequest } from "@/types/users";
import { apiFetch } from "@/utils/api";

export const editUser = async (
  userId: number,
  user: PutUserRequest
): Promise<PutUserReponse> => {
  return apiFetch(`/users/${userId}`, {
    method: "PUT",
    body: JSON.stringify(user),
  });
};