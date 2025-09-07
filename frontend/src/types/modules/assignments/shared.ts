import type { Timestamp } from "@/types/common";

export const ASSIGNMENT_TYPES = ['assignment', 'practical'] as const;
export type AssignmentType = (typeof ASSIGNMENT_TYPES)[number];

export const ASSIGNMENT_STATUSES = ['setup', 'ready', 'open', 'closed', 'archived'] as const;
export type AssignmentStatus = (typeof ASSIGNMENT_STATUSES)[number];

export const ASSIGNMENT_FILE_TYPES = ['spec', 'main', 'memo', 'makefile', 'mark_allocator', 'config'] as const;
export type AssignmentFileType =(typeof ASSIGNMENT_FILE_TYPES)[number];

export interface Assignment extends Timestamp {
  id: number;
  module_id: number;
  name: string;
  description: string;
  assignment_type: AssignmentType;
  available_from: string; // ISO
  due_date: string;
  status: AssignmentStatus,
}


export interface AssignmentFile extends Timestamp {
  id: number;
  assignment_id: number;
  filename: string;
  path: string;
  file_type: AssignmentFileType;
}

export interface AssignmentReadiness {
  config_present: boolean;
  tasks_present: boolean;
  main_present: boolean;
  memo_present: boolean;
  makefile_present: boolean;
  memo_output_present: boolean;
  mark_allocator_present: boolean;
  is_ready: boolean;
}

export interface BestMark {
  earned: number;
  total: number;
  attempt: number;
  submission_id: number;
}

export interface AssignmentDetails extends Assignment {
  files: AssignmentFile[];
  best_mark?: BestMark;
}
