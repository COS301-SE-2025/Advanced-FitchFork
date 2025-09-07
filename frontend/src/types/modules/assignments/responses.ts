import type { Assignment, AssignmentDetails, AssignmentFile, AssignmentReadiness } from ".";
import type { ApiResponse, PaginationResponse } from "@/types/common";

// ─────────────────────────────────────────────────────────────
// GET Responses Types
// ─────────────────────────────────────────────────────────────

export type GetListAssignmentsResponse = ApiResponse<{
  assignments: Assignment[];
} & PaginationResponse>;

export type GetAssignmentResponse = ApiResponse<AssignmentDetails>

export type GetListAssignmentFilesResponse = ApiResponse<AssignmentFile[]>;

export type GetAssignmentReadinessResponse = ApiResponse<AssignmentReadiness>;

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