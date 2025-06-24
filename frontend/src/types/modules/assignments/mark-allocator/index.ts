import type { ApiResponse } from "@/types/common";

export interface SubsectionAllocator {
  name: string;
  value: number;
}

export interface TaskAllocator {
  name: string;
  value: number;
  subsections?: SubsectionAllocator[];
}

// Each task is keyed by a dynamic label (e.g., "task1", "task2")
export type MarkAllocatorItem = {
  [taskKey: string]: TaskAllocator;
};

// The full response structure
export type GetMarkAllocatorResponse = ApiResponse<MarkAllocatorItem[]>;
