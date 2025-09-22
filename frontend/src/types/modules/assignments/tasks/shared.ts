import type { Timestamp } from "@/types/common";

export interface Task extends Timestamp {
  id: number;
  assignment_id: number;
  task_number: number;
  name: string;
  code_coverage: boolean;
  command: string;
}

export interface SubsectionDetail {
  name: string;
  value: number;
  memo_output: string | null;
  feedback?: string;
  regex?: string[];
}