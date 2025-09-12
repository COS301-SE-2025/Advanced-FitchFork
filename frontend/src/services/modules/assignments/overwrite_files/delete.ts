import type { ApiResponse } from "@/types/common";
import { api } from "@/utils/api";

/** Delete all overwrite files for a given task. */
export const deleteOverwriteFiles = async (
  moduleId: number,
  assignmentId: number,
  taskId: number
): Promise<ApiResponse<{}>> => {
  return api.delete<{}>(
    `/modules/${moduleId}/assignments/${assignmentId}/overwrite_files/task/${taskId}`
  );
};
