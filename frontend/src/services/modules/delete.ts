import type { DeleteModuleResponse, DeletePersonnelResponse } from "@/types/modules";
import { apiFetch } from "@/utils/api";

export const deleteModule = async (
  moduleId: number
): Promise<DeleteModuleResponse> => {
  return apiFetch(`/modules/${moduleId}`, { method: "DELETE" });
};

export const removeLecturers = async (
  moduleId: number,
  payload: { user_ids: number[] }
): Promise<DeletePersonnelResponse> => {
  return apiFetch(`/modules/${moduleId}/lecturers`, {
    method: "DELETE",
    body: JSON.stringify(payload),
  });
};

export const removeTutors = async (
  moduleId: number,
  payload: { user_ids: number[] }
): Promise<DeletePersonnelResponse> => {
  return apiFetch(`/modules/${moduleId}/tutors`, {
    method: "DELETE",
    body: JSON.stringify(payload),
  });
};

export const removeStudents = async (
  moduleId: number,
  payload: { user_ids: number[] }
): Promise<DeletePersonnelResponse> => {
  return apiFetch(`/modules/${moduleId}/students`, {
    method: "DELETE",
    body: JSON.stringify(payload),
  });
};