import type { Assignment, AssignmentFile } from ".";
import type { ApiResponse } from "@/types/common";

// ─────────────────────────────────────────────────────────────
// POST Responses Types
// ─────────────────────────────────────────────────────────────

export type PostAssignmentResponse = ApiResponse<Assignment>;

export type PostUploadAssignmentFileResonse = ApiResponse<AssignmentFile>;


// ─────────────────────────────────────────────────────────────
// PUT Responses Types
// ─────────────────────────────────────────────────────────────

export type PutAssignmentResponse = ApiResponse<Assignment>;

export type BulkUpdateAssignmentsResponse = ApiResponse<{
  updated: number;
  failed: { id: number; error: string }[];
}>;

// ─────────────────────────────────────────────────────────────
// DELETE Responses Types
// ─────────────────────────────────────────────────────────────

export type DeleteAssignmentResponse = ApiResponse<null>;

export type DeleteAssignmentFilesResponse = ApiResponse<{ not_found: string[];} | null>;

export type BulkDeleteAssignmentsResponse = ApiResponse<{
  deleted: number;
  failed: { id: number; error: string }[];
}>;