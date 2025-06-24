import type { Assignment } from ".";

// ─────────────────────────────────────────────────────────────
// Utility Types
// ─────────────────────────────────────────────────────────────

// Used for post and put routes
type AssignmentPayload = Omit<Assignment, "id" | "module_id" | "created_at" | "updated_at">;

// ─────────────────────────────────────────────────────────────
// POST Request Types
// ─────────────────────────────────────────────────────────────

export type PostAssignmentRequest = AssignmentPayload;

// ─────────────────────────────────────────────────────────────
// PUT Request Types
// ─────────────────────────────────────────────────────────────

export type PutAssignmentRequest = Partial<AssignmentPayload>;