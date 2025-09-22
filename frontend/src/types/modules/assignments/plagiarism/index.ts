import type { ApiResponse, PaginationResponse, Timestamp } from "@/types/common";

// ------------ shared ------------
export const PLAGIARISM_CASE_STATUS_TYPES = ["review", "flagged", "reviewed"] as const;
export type PlagiarismCaseStatus = (typeof PLAGIARISM_CASE_STATUS_TYPES)[number];

export interface PlagiarismCase extends Timestamp {
  id: number;
  assignment_id: number;
  submission_id_1: number;
  submission_id_2: number;
  description: string;
  status: PlagiarismCaseStatus;
  similarity: number;
  lines_matched: number;
  report_id: number | null;
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
  lines_matched: number;
  report_id: number | null;
  created_at: string;
  updated_at: string;
  submission_1: SubmissionLite;
  submission_2: SubmissionLite;
}

export interface PlagiarismCaseListData extends PaginationResponse {
  cases: PlagiarismCaseItem[];
}

export type GetListPlagiarismCasesResponse = ApiResponse<PlagiarismCaseListData>;

// ------------ graph ------------
export interface PlagiarismGraphLink {
  source: string;
  target: string;
  case_id: number;
  report_id?: number,
  similarity: number;
  lines_matched: number;
  status: PlagiarismCaseStatus;
}

export type GetPlagiarismGraphResponse = ApiResponse<{
  links: PlagiarismGraphLink[];
}>;

// ------------ moss report(s) ------------
export const MOSS_FILTER_MODES = ["all", "whitelist", "blacklist"] as const;
export type MossFilterMode = (typeof MOSS_FILTER_MODES)[number];

export interface MossReport {
  id: number;
  report_url: string;
  generated_at: string; // RFC 3339
  has_archive: boolean;
  archive_generated_at: string | null;
  filter_mode: MossFilterMode;
  filter_patterns: string[] | null;
  description: string;
}

// list
export type MossReportListResponse = ApiResponse<{
  reports: MossReport[];
}>;

export interface HashScanPayload {
  /** When true, auto-create plagiarism cases for each unique pair in a collision group. */
  create_cases?: boolean;
}

export interface HashScanSubmissionInfo {
  submission_id: number;
  user_id: number;
  attempt: number;
  filename: string;
  /** RFC 3339 */
  created_at: string;
}

export interface HashScanCollisionGroup {
  file_hash: string;
  submissions: HashScanSubmissionInfo[];
}

export interface HashScanCreatedCases {
  created: PlagiarismCase[];   // server returns full cases (matches your example)
  skipped_existing: number;
}

export interface HashScanData {
  assignment_id: number;
  /** "Best" | "Last" as sent by backend; keep string to be future-proof */
  policy_used: string;
  group_count: number;
  groups: HashScanCollisionGroup[];
  cases: HashScanCreatedCases;
}