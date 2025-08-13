import type { ApiResponse } from "@/types/common";
import type { MarkAllocatorFile } from "@/types/modules/assignments/mark-allocator";
import { apiFetch } from "@/utils/api";

export const updateMarkAllocator = async (
  moduleId: number,
  assignmentId: number,
  payload: MarkAllocatorFile
): Promise<ApiResponse<null>> => {
  return apiFetch(`/modules/${moduleId}/assignments/${assignmentId}/mark_allocator`, {
    method: "PUT",
    headers: { "Content-Type": "application/json" }, // ensure JSON parse
    body: JSON.stringify(payload),
  });
};
