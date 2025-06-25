import type { Assignment } from ".";

// ─────────────────────────────────────────────────────────────
// Utility Types
// ─────────────────────────────────────────────────────────────

// Used for post and put routes
type AssignmentPayload = Omit<Assignment, "id" | "module_id" | "created_at" | "updated_at">;

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