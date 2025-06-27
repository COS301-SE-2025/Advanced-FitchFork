import type { Timestamp } from "@/types/common";

export type AssignmentType = 'Assignment' | 'Practical';
export const ASSIGNMENT_TYPES: AssignmentType[] = ['Assignment', 'Practical'];

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
  status?: string,
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
