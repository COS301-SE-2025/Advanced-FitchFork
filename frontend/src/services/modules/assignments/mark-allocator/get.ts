
import type { GetMarkAllocatorResponse } from "@/types/modules/assignments/mark-allocator";
import { apiFetch } from "@/utils/api";

export const getMarkAllocator = async (
  moduleId: number,
  assignmentId: number
): Promise<GetMarkAllocatorResponse> => {
  return apiFetch(`/modules/${moduleId}/assignments/${assignmentId}/mark-allocator`);
};
