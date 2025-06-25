import type { Timestamp } from "@/types/common";

export type AssignmentType = 'Assignment' | 'Practical';
export const ASSIGNMENT_TYPES: AssignmentType[] = ['Assignment', 'Practical'];

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
}