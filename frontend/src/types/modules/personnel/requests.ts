import type { PaginationRequest } from "@/types/common";
import type { ModuleRole } from "@/types/modules"

// ─────────────────────────────────────────────────────────────
// POST Request Types
// ─────────────────────────────────────────────────────────────

export interface PostAssignPersonnelRequest {
  user_ids: number[];
  role: ModuleRole;
}

// ─────────────────────────────────────────────────────────────
// GET Request Types
// ─────────────────────────────────────────────────────────────

export interface GetPersonnelQuery extends PaginationRequest {
  role: ModuleRole;
  email?: string;
  username?: string;
}

export interface GetEligibleUsersQuery extends PaginationRequest {
  email?: string;
  username?: string;
}

// ─────────────────────────────────────────────────────────────
// DELETE Request Types
// ─────────────────────────────────────────────────────────────

export interface DeletePersonnelRequest {
  user_ids: number[];
  role: ModuleRole;
}
