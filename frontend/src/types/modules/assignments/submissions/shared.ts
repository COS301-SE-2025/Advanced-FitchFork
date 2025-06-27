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
  tasks?: TaskBreakdown[];
  code_coverage?: CodeCoverageEntry[];
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
}

export interface TaskBreakdown {
  task_number: number;
  name: string;
  score: SubmissionMark;
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
  username: string;
}