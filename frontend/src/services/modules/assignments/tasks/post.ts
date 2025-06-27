import type { PostTaskRequest, PostTaskResponse } from "@/types/modules/assignments/tasks";
import { apiFetch } from "@/utils/api";

export const createTask = async (
  moduleId: number,
  assignmentId: number,
  data: PostTaskRequest
): Promise<PostTaskResponse> => {
  return apiFetch(`/modules/${moduleId}/assignments/${assignmentId}/tasks`, {
    method: "POST",
    body: JSON.stringify(data),
  });
};