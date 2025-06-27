import type { PutEditTaskResponse } from "@/types/modules/assignments/tasks";
import { apiFetch } from "@/utils/api";

export const editTask = async (
  moduleId: number,
  assignmentId: number,
  taskId: number,
  payload: {
    name: string,
    command: string,
  }
): Promise<PutEditTaskResponse> => {
  return apiFetch(`/modules/${moduleId}/assignments/${assignmentId}/tasks/${taskId}`,
    { 
      method: "PUT", 
      body: JSON.stringify(payload) 
    }
  );
}