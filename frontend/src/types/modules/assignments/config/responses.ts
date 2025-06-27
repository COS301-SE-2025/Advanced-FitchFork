import type { ApiResponse } from "@/types/common";
import type { AssignmentConfig } from "./shared";

// ─────────────────────────────────────────────────────────────
// GET Responses Types
// ─────────────────────────────────────────────────────────────

export type GetAssignmentConfigResponse = ApiResponse<AssignmentConfig>;

// ─────────────────────────────────────────────────────────────
// POST Responses Types
// ─────────────────────────────────────────────────────────────

export type PostAssignmentConfigResponse = ApiResponse<AssignmentConfig>;

// ─────────────────────────────────────────────────────────────
// PUT Responses Types
// ─────────────────────────────────────────────────────────────

export type UpdateAssignmentConfigResponse = ApiResponse<null>;