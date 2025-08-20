import type { ApiResponse, PaginationResponse } from "@/types/common";
import type { Submission } from "./shared";

// ─────────────────────────────────────────────────────────────
// GET Responses Types
// ─────────────────────────────────────────────────────────────

export type GetSubmissionDetailResponse = ApiResponse<Submission>;

export type GetSubmissionListResponse = ApiResponse<{
  submissions: Submission[];
} & PaginationResponse>;


// ─────────────────────────────────────────────────────────────
// POST Responses Types
// ─────────────────────────────────────────────────────────────

export type PostSubmitAssignmentResponse = ApiResponse<Submission>;

export interface FailedRemark {
  id?: number;
  error: string;
}

export interface RemarkResponse {
  regraded: number;
  failed: FailedRemark[];
}