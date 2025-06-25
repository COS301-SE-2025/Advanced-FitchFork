import type { ApiResponse, PaginationResponse } from "@/types/common";
import type { User } from ".";
import type { Module, ModuleRole } from "@/types/modules";

// ─────────────────────────────────────────────────────────────
// GET Response Types
// ─────────────────────────────────────────────────────────────

export type GetListUsersResponse = ApiResponse<{
  users: User[];
} & PaginationResponse>;

export type GetUserResponse = ApiResponse<User>;

export type GetUserModulesReponse = ApiResponse<({
  role: ModuleRole
} & Module)[]>;

// ─────────────────────────────────────────────────────────────
// PUT Response Types
// ─────────────────────────────────────────────────────────────

export type PutUserReponse = ApiResponse<User>;

export type DeleteUserResponse = ApiResponse<null>;