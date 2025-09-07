import type { PostGenerateMarkAllocatorResponse } from "@/types/modules/assignments/mark-allocator";
import { apiFetch } from "@/utils/api";

export const generateMarkAllocator = async (
  moduleId: number,
  assignmentId: number
): Promise<PostGenerateMarkAllocatorResponse> => {
  return apiFetch(`/modules/${moduleId}/assignments/${assignmentId}/mark_allocator/generate`, 
    { method: "POST" }
  );
}