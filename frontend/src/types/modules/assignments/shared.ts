import type { Timestamp } from "@/types/common";
import type { SubmissionMode, GradingPolicy, LateOptions } from "./config";

export const ASSIGNMENT_TYPES = ['assignment', 'practical'] as const;
export type AssignmentType = (typeof ASSIGNMENT_TYPES)[number];

export const ASSIGNMENT_STATUSES = ['setup', 'ready', 'open', 'closed', 'archived'] as const;
export type AssignmentStatus = (typeof ASSIGNMENT_STATUSES)[number];

export const ASSIGNMENT_FILE_TYPES = ['spec', 'main', 'memo', 'makefile', 'mark_allocator', 'config'] as const;
export type AssignmentFileType = (typeof ASSIGNMENT_FILE_TYPES)[number];

export interface Assignment extends Timestamp {
  id: number;
  module_id: number;
  name: string;
  description: string;
  assignment_type: AssignmentType;
  available_from: string; // ISO
  due_date: string;       // ISO
  status: AssignmentStatus;
}

export interface AssignmentFile extends Timestamp {
  id: number;
  assignment_id: number;
  filename: string;
  path: string;
  file_type: AssignmentFileType;
}

export interface AssignmentReadiness {
  submission_mode: SubmissionMode;
  config_present: boolean;
  tasks_present: boolean;
  main_present: boolean;
  interpreter_present: boolean;
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

export interface AttemptsInfo {
  used: number;
  max: number | null;        // null => unlimited
  remaining: number | null;  // null => unlimited
  can_submit: boolean;
  limit_attempts: boolean;
}

export interface AssignmentPolicy {
  allow_practice_submissions: boolean;
  submission_mode: SubmissionMode;
  grading_policy: GradingPolicy;
  limit_attempts: boolean;
  pass_mark: number;
  late: LateOptions;
}
export interface AssignmentDetails {
  assignment: Assignment;
  files: AssignmentFile[];
  policy: AssignmentPolicy;
  best_mark?: BestMark;        // only for students
  attempts?: AttemptsInfo;     // only for students
}

export interface AssignmentStats {
  // headline
  total: number;
  graded: number;
  pending: number;
  pass_rate: number;   // %
  avg_mark: number;    // %
  median: number;      // %
  p75: number;         // %
  stddev: number;
  best: number;        // %
  worst: number;       // %

  // new headline extras
  total_marks: number;             // sum of all total marks across submissions
  num_students_submitted: number;  // distinct students with ≥1 submission
  num_passed: number;
  num_failed: number;
  num_full_marks: number;

  // flags
  late: number;
  on_time: number;
  ignored: number;
}
