import type { PutEditTaskResponse, TaskType } from "@/types/modules/assignments/tasks";
import { apiFetch } from "@/utils/api";

export const editTask = async (
  moduleId: number,
  assignmentId: number,
  taskId: number,
  payload: {
    name: string,
    command: string,
    task_type: TaskType
  }
): Promise<PutEditTaskResponse> => {
  return apiFetch(`/modules/${moduleId}/assignments/${assignmentId}/tasks/${taskId}`,
    { 
      method: "PUT", 
      body: JSON.stringify(payload) 
    }
  );
}