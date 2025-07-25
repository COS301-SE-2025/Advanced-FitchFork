import type { Timestamp } from "@/types/common";

export const ASSIGNMENT_TYPES = ['assignment', 'practical'] as const;
export type AssignmentType = (typeof ASSIGNMENT_TYPES)[number];

export const ASSIGNMENT_STATUSES = ['setup', 'ready', 'open', 'closed', 'archived'] as const;
export type AssignmentStatus = (typeof ASSIGNMENT_STATUSES)[number];

export type FileType =
  | 'spec'
  | 'main'
  | 'memo'
  | 'makefile'
  | 'mark_allocator'
  | 'config';

export const FILE_TYPES: FileType[] = [
  'spec',
  'main',
  'memo',
  'makefile',
  'mark_allocator',
  'config',
];

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
  file_type: FileType;
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
