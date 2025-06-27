import type { User } from "@/types/users";
import type {Module, ModuleRole} from "./shared";
import type { ApiResponse, PaginationResponse } from "../common";

export interface ListLecturersResponse { users: User[]; }
export interface ListTutorsResponse { users: User[]; }
export interface ListStudentsResponse { users: User[]; }

// ─────────────────────────────────────────────────────────────
// GET Response Types
// ─────────────────────────────────────────────────────────────

export type GetListModulesResponse = ApiResponse<{
  modules: Module[];
} & PaginationResponse>;

export type GetModuleResponse = ApiResponse<{
  lecturers: User[];
  tutors: User[];
  students: User[];
} & Module>;

export type GetMyModulesResponse = ApiResponse<{
  as_student: Module[];
  as_tutor: Module[];
  as_lecturer: Module[];
}>;

export type GetPersonnelReponse = ApiResponse<{ 
  users: User[];
} & PaginationResponse>;

export type GetModulesForUserResponse = ApiResponse<{
 role: ModuleRole
} & Module[]>;

export type GetEligibleUsersResponse = ApiResponse<{
  users: User[];
} & PaginationResponse>;

// ─────────────────────────────────────────────────────────────
// POST Response Types
// ─────────────────────────────────────────────────────────────

export type PostModuleResponse = ApiResponse<Module>;
export type PostAssignPersonnelResponse = ApiResponse<null>;

// ─────────────────────────────────────────────────────────────
// PUT Response Types
// ─────────────────────────────────────────────────────────────

export type PutModuleResponse = ApiResponse<Module>;

// ─────────────────────────────────────────────────────────────
// DELETE Response Types
// ─────────────────────────────────────────────────────────────

export type DeleteModuleResponse = ApiResponse<null>;
export type DeletePersonnelResponse = ApiResponse<null>;