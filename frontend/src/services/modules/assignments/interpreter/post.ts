import type { ApiResponse } from "@/types/common";
import type { InterpreterInfo } from "@/types/modules/assignments/interpreter";
import { apiUpload } from "@/utils/api";

export async function uploadInterpreter(
  moduleId: number,
  assignmentId: number,
  file: File,
  command: string
): Promise<ApiResponse<InterpreterInfo>> {
  const form = new FormData();
  form.append("command", command);
  form.append("file", file);
  return apiUpload<InterpreterInfo>(`/modules/${moduleId}/assignments/${assignmentId}/interpreter`, form);
}
