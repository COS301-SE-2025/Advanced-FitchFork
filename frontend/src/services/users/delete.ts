import type { DeleteUserResponse } from "@/types/users";
import { apiFetch } from "@/utils/api";

export const deleteUser = async (
  userId: number
): Promise<DeleteUserResponse> => {
  return apiFetch(`/users/${userId}`, { method: "DELETE" });
};