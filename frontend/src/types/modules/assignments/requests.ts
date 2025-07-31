import type { Assignment } from ".";

// ─────────────────────────────────────────────────────────────
// Utility Types
// ─────────────────────────────────────────────────────────────

// Used for post and put routes
type AssignmentPayload = Omit<Assignment, "id" | "module_id" | "status" | "created_at" | "updated_at">;

// ─────────────────────────────────────────────────────────────
// POST Request Types
// ─────────────────────────────────────────────────────────────

export interface RegisterRequest {
  username: string;
  email: string;
  password: string;
}

export interface LoginRequest {
  username: string;
  password: string;
}

export type PostAssignmentRequest = AssignmentPayload;

// ─────────────────────────────────────────────────────────────
// PUT Request Types
// ─────────────────────────────────────────────────────────────

export type PutAssignmentRequest = Partial<AssignmentPayload>;

export interface BulkUpdateAssignmentsRequest {
  assignment_ids: number[];
  available_from?: string;
  due_date?: string;
}

// ─────────────────────────────────────────────────────────────
// DELETE Request Types
// ─────────────────────────────────────────────────────────────

export interface BulkDeleteAssignmentsRequest {
  assignment_ids: number[];
}