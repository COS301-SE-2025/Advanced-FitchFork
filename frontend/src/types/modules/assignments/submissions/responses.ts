import type { ApiResponse } from "@/types/common";
import type { Submission } from "./shared";
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