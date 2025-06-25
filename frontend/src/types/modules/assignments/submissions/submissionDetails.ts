import type { Score, Timestamp, ApiResponse } from "@/types/common";


export interface SubmissionMark {
  earned: number;
  total: number;
  percentage: number;
}

export interface SubsectionBreakdown extends Score {
  label: string;
  status: string;
}

export interface TaskBreakdown {
  task_number: number;
  name: string;
  score: Score;
  feedback: string;
  subsections: SubsectionBreakdown[];
}

export interface CodeCoverageEntry {
  class: string;
  percentage: number;
}

export interface SubmissionUserInfo {
  id: number;
  email: string;
  student_number: string;
}

export interface SubmissionDetailBase extends Timestamp {
  id: number;
  attempt: number;
  filename: string;
  hash: string;
  mark: SubmissionMark;
  is_practice: boolean;
  is_late: boolean;
  tasks: TaskBreakdown[];
  code_coverage: CodeCoverageEntry[];
}

// For staff (includes user info)
export interface SubmissionDetailForStaff extends SubmissionDetailBase {
  user: SubmissionUserInfo;
}

// For students (no user field)
export type SubmissionDetailForStudent = SubmissionDetailBase;

// Union response type
export type GetSubmissionDetailResponse = ApiResponse<
  SubmissionDetailForStaff | SubmissionDetailForStudent
>;
