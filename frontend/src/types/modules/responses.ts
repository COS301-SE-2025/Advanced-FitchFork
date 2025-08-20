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
  as_lecturer: Module[];  
  as_assistant_lecturer: Module[];
  as_student: Module[];
  as_tutor: Module[];
}>;

export type GetModulesForUserResponse = ApiResponse<{
 role: ModuleRole
} & Module[]>;

// ─────────────────────────────────────────────────────────────
// POST Response Types
// ─────────────────────────────────────────────────────────────

export type PostModuleResponse = ApiResponse<Module>;

// ─────────────────────────────────────────────────────────────
// PUT Response Types
// ─────────────────────────────────────────────────────────────

export type PutModuleResponse = ApiResponse<Module>;

// ─────────────────────────────────────────────────────────────
// DELETE Response Types
// ─────────────────────────────────────────────────────────────

export type DeleteModuleResponse = ApiResponse<null>;
export type DeletePersonnelResponse = ApiResponse<null>;