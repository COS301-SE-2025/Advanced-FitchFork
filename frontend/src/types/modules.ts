import type { SortOption, Timestamp } from "./common";
import type { User } from "./users";

// ─────────────────────────────────────────────────────────────
// CORE ENTITY
// ─────────────────────────────────────────────────────────────

export type ModuleRole = "Lecturer" | "Tutor" | "Student";
export const MODULE_ROLES: ModuleRole[] = ['Lecturer', 'Tutor', 'Student'];

/**
 * Base module entity returned from the API.
 */
export interface Module extends Timestamp {
  readonly id: number;
  code: string;
  year: number;
  description: string;
  credits: number;
}

/**
 * Payload for creating or editing a module.
 * Omits server-managed fields.
 */
export type ModulePayload = Omit<Module, "id" | "created_at" | "updated_at">;

// ─────────────────────────────────────────────────────────────
// MODULE DETAILS RESPONSE
// ─────────────────────────────────────────────────────────────

/**
 * Response for fetching a module and its personnel.
 */
export interface ModuleDetailsResponse extends Module {
  lecturers: User[];
  tutors: User[];
  students: User[];
}

// ─────────────────────────────────────────────────────────────
// MODULE LISTING AND FILTERING
// ─────────────────────────────────────────────────────────────

/**
 * Request parameters for listing and filtering modules.
 */
export interface ListModulesRequest {
  page: number;
  per_page: number;
  sort?: SortOption[];
  query?: string;
  code?: string;
  year?: number;
}

/**
 * Paginated response of modules.
 */
export interface ListModulesResponse {
  modules: Module[];
  page: number;
  per_page: number;
  total: number;
}

/**
 * Modules grouped by the current user's role.
 */
export interface MyModulesResponse {
  as_student: Module[];
  as_tutor: Module[];
  as_lecturer: Module[];
}

// ─────────────────────────────────────────────────────────────
// MODULE PERSONNEL MANAGEMENT
// ─────────────────────────────────────────────────────────────

export interface AssignLecturersRequest { user_ids: number[]; }
export interface RemoveLecturersRequest { user_ids: number[]; }
export interface AssignTutorsRequest { user_ids: number[]; }
export interface RemoveTutorsRequest { user_ids: number[]; }
export interface EnrollStudentsRequest { user_ids: number[]; }
export interface RemoveStudentsRequest { user_ids: number[]; }

export interface ListLecturersResponse { users: User[]; }
export interface ListTutorsResponse { users: User[]; }
export interface ListStudentsResponse { users: User[]; }

// ─────────────────────────────────────────────────────────────
// MODULES FOR A SPECIFIC USER
// ─────────────────────────────────────────────────────────────

/**
 * A module and the user's role in it.
 */
export interface UserModuleRole {
  readonly id: number;
  code: string;
  year: number;
  description: string;
  credits: number;
  role: "Lecturer" | "Tutor" | "Student";
  created_at: string;
  updated_at: string;
}
