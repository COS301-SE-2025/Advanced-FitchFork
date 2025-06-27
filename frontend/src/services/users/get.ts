import type { PaginationRequest } from "@/types/common";
import type { GetListUsersResponse, GetUserResponse, GetUserModulesReponse } from "@/types/users";
import { apiFetch, buildQuery } from "@/utils/api";

export const listUsers = async (
  options: {
    email?: string;
    username?: string;
    admin?: boolean;
  } & PaginationRequest
): Promise<GetListUsersResponse> => {
  const query = buildQuery(options);
  return apiFetch(`/users?${query}`);
};

export const getUser = async (
  userId: number
): Promise<GetUserResponse> => {
  return apiFetch(`/users/${userId}`);
};

export const getUserModules = async (
  userId: number
): Promise<GetUserModulesReponse> => {
  return apiFetch(`/users/${userId}/modules`);
};