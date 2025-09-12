import type { ApiResponse } from "@/types/common";
import { apiUpload } from "@/utils/api";

/** Upload one or more overwrite files (multipart/form-data). */
export const uploadOverwriteFiles = async (
  moduleId: number,
  assignmentId: number,
  taskId: number,
  files: File[] | FileList
): Promise<ApiResponse<string[]>> => {
  const form = new FormData();
  const arr = Array.from(files as File[]); // supports FileList or File[]

  if (arr.length === 0) {
    return { success: false, data: [] as string[], message: "No files selected." };
  }

  for (const f of arr) form.append("file", f, f.name);

  return apiUpload<string[]>(
    `/modules/${moduleId}/assignments/${assignmentId}/overwrite_files/task/${taskId}`,
    form
  );
};
