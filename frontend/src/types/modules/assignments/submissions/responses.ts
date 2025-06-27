import type { ApiResponse, PaginationResponse } from "@/types/common";
import type { Submission } from "./shared";

// ─────────────────────────────────────────────────────────────
// GET Responses Types
// ─────────────────────────────────────────────────────────────

export type GetSubmissionDetailResponse = ApiResponse<Submission>;

export type GetSubmissionListResponse = ApiResponse<
  ({
    submissions: Submission[];
  } & PaginationResponse)
  |
  Submission[]
>;


// ─────────────────────────────────────────────────────────────
// POST Responses Types
// ─────────────────────────────────────────────────────────────

export type PostSubmitAssignmentResponse = ApiResponse<Submission>;