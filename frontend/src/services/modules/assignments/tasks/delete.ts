import type { DeleteTaskResponse } from "@/types/modules/assignments/tasks";
import { apiFetch } from "@/utils/api";

export const deleteTask = async (
  moduleId: number,
  assignmentId: number,
  taskId: number
): Promise<DeleteTaskResponse> => {
  return apiFetch(`/modules/${moduleId}/assignments/${assignmentId}/tasks/${taskId}`,
    { method: "DELETE" }
  );
}