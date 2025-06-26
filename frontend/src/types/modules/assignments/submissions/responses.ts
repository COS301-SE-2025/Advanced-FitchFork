import type { ApiResponse, PaginationResponse } from "@/types/common";
import type { Submission, SubmissionDetail, SubmissionUserInfo } from "./shared";

// ─────────────────────────────────────────────────────────────
// GET Responses Types
// ─────────────────────────────────────────────────────────────

export type GetSubmissionDetailResponse = ApiResponse<
  (
    ({user: SubmissionUserInfo} & SubmissionDetail) // For Student
    |
    SubmissionDetail                                // For Staff
  )
>;

export type GetSubmissionListResponse = ApiResponse<
  (
    ({user: SubmissionUserInfo} & Submission) // For Student
    |
    Submission                                // For Staff
  ) & PaginationResponse
>;

// ─────────────────────────────────────────────────────────────
// POST Responses Types
// ─────────────────────────────────────────────────────────────

export type PostSubmitAssignmentResponse = ApiResponse<SubmissionDetail>;