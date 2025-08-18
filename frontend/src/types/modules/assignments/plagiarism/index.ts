import type { ApiResponse, PaginationResponse, Timestamp } from "@/types/common";

export const PLAGIARISM_CASE_STATUS_TYPES = ['review', 'flagged', 'reviewed'] as const;
export type PlagiarismCaseStatus = (typeof PLAGIARISM_CASE_STATUS_TYPES)[number];

export interface PlagiarismCase extends Timestamp {
  id: number;               // BIGINT
  assignment_id: number;    // FK -> assignments.id
  submission_id_1: number;  // FK -> assignment_submissions.id
  submission_id_2: number;  // FK -> assignment_submissions.id
  description: string;
  status: PlagiarismCaseStatus;
  similarity: number;
}

export interface UserLite {
  id: number;
  username: string;
  email: string;
  profile_picture_path: string | null;
}

export interface SubmissionLite {
  id: number;
  filename: string;
  created_at: string;
  user: UserLite;
}

export interface PlagiarismCaseItem {
  id: number;
  status: PlagiarismCaseStatus;
  description: string;
  similarity: number;
  created_at: string;
  updated_at: string;
  submission_1: SubmissionLite;
  submission_2: SubmissionLite;
}

export interface PlagiarismCaseListData extends PaginationResponse {
  cases: PlagiarismCaseItem[];
}

export type GetListPlagiarismCasesResponse = ApiResponse<PlagiarismCaseListData>;

export interface PlagiarismGraphLink {
  source: string;
  target: string;
}

export type GetPlagiarismGraphResponse = ApiResponse<{
  links: PlagiarismGraphLink[];
}>;

export type GetMossReportResponse = ApiResponse<{
  report_url: string;
  generated_at: string;
}>;
