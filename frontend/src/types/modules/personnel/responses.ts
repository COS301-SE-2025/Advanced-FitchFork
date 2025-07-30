import type { ApiResponse, PaginationResponse } from "@/types/common";
import type { MinimalUser } from "./shared";

// ─────────────────────────────────────────────────────────────
// POST Response Types
// ─────────────────────────────────────────────────────────────

export type PostAssignPersonnelResponse = ApiResponse<null>;

// ─────────────────────────────────────────────────────────────
// GET Response Types
// ─────────────────────────────────────────────────────────────

export type GetPersonnelResponse = ApiResponse<{
  users: MinimalUser[];
} & PaginationResponse>;

export type GetEligibleUsersResponse = ApiResponse<{
  users: MinimalUser[];
} & PaginationResponse>;

// ─────────────────────────────────────────────────────────────
// DELETE Response Types
// ─────────────────────────────────────────────────────────────

export type DeletePersonnelResponse = ApiResponse<null>;