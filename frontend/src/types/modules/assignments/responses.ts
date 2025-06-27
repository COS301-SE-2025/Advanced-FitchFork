import type { Assignment, AssignmentFile, AssignmentReadiness } from ".";
import type { ApiResponse, PaginationResponse } from "@/types/common";

// ─────────────────────────────────────────────────────────────
// GET Responses Types
// ─────────────────────────────────────────────────────────────

export type GetListAssignmentsResponse = ApiResponse<{
  assignments: Assignment[];
} & PaginationResponse>;

export type GetAssignmentResponse = ApiResponse<{
  files: AssignmentFile[];
} & Assignment>

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

// ─────────────────────────────────────────────────────────────
// DELETE Responses Types
// ─────────────────────────────────────────────────────────────

export type DeleteAssignmentResponse = ApiResponse<null>;

export type DeleteAssignmentFilesResponse = ApiResponse<{ not_found: string[];} | null>;