import type { ApiResponse } from "@/types/common"; // adjust the import path as needed

export interface MemoSubsection {
  label: string;
  output: string;
}

export interface MemoTaskOutput {
  task_number: number;
  name: string;
  subsections: MemoSubsection[];
}

export interface MemoOutputData {
  tasks: MemoTaskOutput[];
}

// Final API response type
export type GetMemoOutputResponse = ApiResponse<MemoOutputData>;
