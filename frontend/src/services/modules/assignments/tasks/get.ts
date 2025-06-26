
import type { GetListTasksResponse, GetTaskResponse } from "@/types/modules/assignments/tasks";
import { apiFetch } from "@/utils/api";

export const listTasks = async (
  moduleId: number,
  assignmentId: number
): Promise<GetListTasksResponse> => {
  return apiFetch(`/modules/${moduleId}/assignments/${assignmentId}/tasks`);
};

export const getTask = async (
  moduleId: number,
  assignmentId: number,
  taskId: number
): Promise<GetTaskResponse> => {
  return apiFetch(`/modules/${moduleId}/assignments/${assignmentId}/tasks/${taskId}`);
}