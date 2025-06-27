import type { PaginationRequest } from "@/types/common";
import type {
  GetEligibleUsersResponse,
  GetListModulesResponse,
  GetModuleResponse,
  GetModulesForUserResponse,
  GetMyModulesResponse, 
  GetPersonnelReponse,
  ModuleRole } from "@/types/modules";
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

export const getLecturers = async (
  moduleId: number,
  options: {
    email?: string;
    username?: string;
  } & PaginationRequest
): Promise<GetPersonnelReponse> => {
  return apiFetch(`/modules/${moduleId}/lecturers?${buildQuery(options)}`);
};

export const getTutors = async (
  moduleId: number,
  options: {
    email?: string;
    username?: string;
  } & PaginationRequest
): Promise<GetPersonnelReponse> => {
  return apiFetch(`/modules/${moduleId}/tutors?${buildQuery(options)}`);
};

export const getStudents = async (
  moduleId: number,
  options: {
    email?: string;
    username?: string;
  } & PaginationRequest
): Promise<GetPersonnelReponse> => {
  return apiFetch(`/modules/${moduleId}/students?${buildQuery(options)}`);
};

export const getModulesForUser = async (
  userId: number
): Promise<GetModulesForUserResponse> => {
  return apiFetch(`/users/${userId}/modules`);
};

export const getEligibleUsersForRole = async (
  moduleId: number,
  role: ModuleRole,
  options: {
    email?: string;
    username?: string;
  } & PaginationRequest
): Promise<GetEligibleUsersResponse> => {
  const query = buildQuery({ ...options, role });
  return apiFetch(`/modules/${moduleId}/eligible-users?${query}`);
};