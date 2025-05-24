import type { SortOption, Timestamp } from "./common";

/**
 * Core user entity with basic public fields and admin flag.
 */
export interface User extends Timestamp {
  id: number;
  student_number: string;
  email: string;
  admin: boolean;
}

/**
 * Only editable fields of a User (used in PUT request payloads).
 */
export type UserEditableFields = Omit<User, "id" | "created_at" | "updated_at">;

/**
 * Paginated user listing request with filtering and sorting.
 */
export interface ListUsersRequest {
  page: number;
  per_page: number;
  sort?: SortOption[];
  query?: string;
  email?: string;
  student_number?: string;
  admin?: boolean;
}

export interface ListUsersResponse {
  users: User[];
  page: number;
  per_page: number;
  total: number;
}

/**
 * User’s involvement in modules, with role info.
 */
export interface ModuleSummary {
  id: number;
  code: string;
  year: number;
  description: string;
  credits: number;
  role: "Lecturer" | "Tutor" | "Student";
  created_at: string;
  updated_at: string;
}

export interface UserModulesResponse {
  modules: ModuleSummary[];
}
