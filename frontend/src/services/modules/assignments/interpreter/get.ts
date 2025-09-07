import type { InterpreterInfo } from "@/types/modules/assignments/interpreter";
import { api, apiDownload } from "@/utils/api";


export async function downloadInterpreter(
  moduleId: number,
  assignmentId: number
): Promise<void> {
  return apiDownload(`/modules/${moduleId}/assignments/${assignmentId}/interpreter`);
}

/** NEW: fetch interpreter metadata */
export async function getInterpreterInfo(
  moduleId: number,
  assignmentId: number
) {
  return api.get<InterpreterInfo>(`/modules/${moduleId}/assignments/${assignmentId}/interpreter/info`);
}