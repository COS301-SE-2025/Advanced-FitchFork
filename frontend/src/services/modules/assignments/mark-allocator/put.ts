import type { ApiResponse } from "@/types/common";
import type { MarkAllocatorItem } from "@/types/modules/assignments/mark-allocator";
import { apiFetch } from "@/utils/api";

export const updateMarkAllocator = async (
  moduleId: number,
  assignmentId: number,
  payload: MarkAllocatorItem[]
): Promise<ApiResponse<null>> => {
  return apiFetch(`/modules/${moduleId}/assignments/${assignmentId}/mark_allocator`, {
    method: "PUT",
    body: JSON.stringify(payload),
    }
  );
}