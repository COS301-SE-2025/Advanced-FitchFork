import type { Score, Timestamp } from "@/types/common";

export interface Submission extends Timestamp {
  id: number;
  attempt: number;
  filename: string;
}

export interface SubmissionMark {
  earned: number;
  total: number;
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
  username: string;
}

export interface SubmissionDetail extends Submission {
  hash: string;
  mark: SubmissionMark;
  is_practice: boolean;
  is_late: boolean;
  tasks: TaskBreakdown[];
  code_coverage: CodeCoverageEntry[];
}
