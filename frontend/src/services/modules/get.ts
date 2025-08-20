import type { PaginationRequest } from "@/types/common";
import type {
  GetListModulesResponse,
  GetModuleResponse,
  GetModulesForUserResponse,
  GetMyModulesResponse } from "@/types/modules";
import { apiFetch, buildQuery } from "@/utils/api";

export const listModules = async (
  options: {
    code?: string;
    year?: number;
  } & PaginationRequest
): Promise<GetListModulesResponse> => {
  return apiFetch(`/modules?${buildQuery(options)}`);
};

export const getModuleDetails = async (
  moduleId: number
): Promise<GetModuleResponse> => {
  return apiFetch(`/modules/${moduleId}`);
};

export const getMyModules = async (): Promise<GetMyModulesResponse> => {
  return apiFetch(`/modules/me`);
};

export const getModulesForUser = async (
  userId: number
): Promise<GetModulesForUserResponse> => {
  return apiFetch(`/users/${userId}/modules`);
};