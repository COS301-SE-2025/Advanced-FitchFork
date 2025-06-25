import type { 
  PostAssignPersonnelRequest,
  PostAssignPersonnelResponse, 
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

export const assignLecturers = async (
  moduleId: number,
  payload: PostAssignPersonnelRequest
): Promise<PostAssignPersonnelResponse> => {
  return apiFetch(`/modules/${moduleId}/lecturers`, {
    method: "POST",
    body: JSON.stringify(payload),
  });
};

export const assignTutors = async (
  moduleId: number,
  payload: PostAssignPersonnelRequest
): Promise<PostAssignPersonnelResponse> => {
  return apiFetch(`/modules/${moduleId}/tutors`, {
    method: "POST",
    body: JSON.stringify(payload),
  });
};

export const enrollStudents = async (
  moduleId: number,
  payload: PostAssignPersonnelRequest
): Promise<PostAssignPersonnelResponse> => {
  return apiFetch(`/modules/${moduleId}/students`, {
    method: "POST",
    body: JSON.stringify(payload),
  });
};