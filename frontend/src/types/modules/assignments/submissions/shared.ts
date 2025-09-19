import type { Timestamp } from "@/types/common";

// ─────────────────────────────────────────────────────────────
// Shared Types
// ─────────────────────────────────────────────────────────────

export interface Submission extends Timestamp {
  id: number;
  attempt: number;
  filename: string;
  hash: string;
  mark: SubmissionMark;
  is_practice: boolean;
  is_late: boolean;
  ignored: boolean;
  score?: number;
  tasks?: TaskBreakdown[];
  code_coverage?: CodeCoverage;
  user?: SubmissionUserInfo;
}

export interface SubmissionMark {
  earned: number;
  total: number;
}

export interface SubsectionBreakdown {
  label: string;
  status: string;
  earned: number;
  total: number;
  feedback?: string; 
}

export interface TaskBreakdown {
  task_number: number;
  name: string;
  score: SubmissionMark;
  subsections: SubsectionBreakdown[];
}

export interface CodeCoverageFile { path: string; earned: number; total: number }
export interface CodeCoverage { summary?: SubmissionMark; files: CodeCoverageFile[] }

export interface SubmissionUserInfo {
  id: number;
  email: string;
  username: string;
}

export interface SubmissionTaskOutput {
  task_number: number;
  raw: string;
}


// Request payload: either specific IDs or all=true (mutually exclusive)
export type ResubmitRequest =
  | { submission_ids: number[]; all?: undefined }
  | { all: true; submission_ids?: undefined };

// Per-submission failure
export interface FailedResubmission {
  id?: number;
  error: string;
}

// Response shape from the endpoint
export interface ResubmitResponse {
  resubmitted: number;
  failed: FailedResubmission[];
}
