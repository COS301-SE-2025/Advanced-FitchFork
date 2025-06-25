import type { GetListTasksResponse } from "@/types/modules/assignments/tasks";
import { apiFetch } from "@/utils/api";

export const listTasks = async (
  moduleId: number,
  assignmentId: number
): Promise<GetListTasksResponse> => {
  return apiFetch(`/modules/${moduleId}/assignments/${assignmentId}/tasks`);
};