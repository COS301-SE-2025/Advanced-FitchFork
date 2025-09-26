import type { Timestamp } from "@/types/common";

export const TASK_TYPES = ['normal', 'coverage', 'valgrind'] as const;
export type TaskType = (typeof TASK_TYPES)[number];

export interface Task extends Timestamp {
  id: number;
  assignment_id: number;
  task_number: number;
  name: string;
  task_type: TaskType;
  command: string;
}

export interface SubsectionDetail {
  name: string;
  value: number;
  memo_output: string | null;
  feedback?: string;
  regex?: string[];
}