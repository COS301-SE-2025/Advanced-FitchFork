
import type {
  Module,
  ModulePayload,
  ListModulesRequest,
  ListModulesResponse,
  ModuleDetailsResponse,
  MyModulesResponse,
  AssignLecturersRequest,
  RemoveLecturersRequest,
  AssignTutorsRequest,
  RemoveTutorsRequest,
  EnrollStudentsRequest,
  RemoveStudentsRequest,
  UserModuleRole
} from "@/types/modules";
import type { User } from "@/types/users";

import { apiFetch, type ApiResponse } from "@/utils/api";

/**
 * ModulesService provides all API operations for managing modules and their associated personnel.
 */
export const ModulesService = {
  /**
   * Fetch a paginated and optionally filtered list of modules.
   * 
   * @param request - Pagination, sorting, and filter options
   * @returns List of modules with pagination metadata
   */
  listModules: (request: ListModulesRequest): Promise<ApiResponse<ListModulesResponse>> => {
    const params = new URLSearchParams();
    params.append("page", request.page.toString());
    params.append("per_page", request.per_page.toString());

    if (request.sort) {
      const sort = request.sort.map(({ field, order }) =>
        order === "descend" ? `-${field}` : field
      ).join(",");
      if (sort) params.append("sort", sort);
    }

    if (request.query) params.append("query", request.query);
    if (request.code) params.append("code", request.code);
    if (typeof request.year === "number") params.append("year", request.year.toString());

    return apiFetch(`/modules?${params.toString()}`, { method: "GET" });
  },

  /**
   * Fetch detailed information about a specific module, including assigned users.
   * 
   * @param moduleId - Module ID to retrieve
   * @returns Full module details with assigned lecturers, tutors, and students
   */
  getModuleDetails: (moduleId: number): Promise<ApiResponse<ModuleDetailsResponse>> =>
    apiFetch(`/modules/${moduleId}`, { method: "GET" }),

  /**
   * Create a new module.
   * 
   * @param payload - Module metadata (code, year, description, credits)
   * @returns The created module
   */
  createModule: (payload: ModulePayload): Promise<ApiResponse<Module>> =>
    apiFetch(`/modules`, {
      method: "POST",
      body: JSON.stringify(payload),
    }),

  /**
   * Update a module's metadata.
   * 
   * @param moduleId - ID of the module to edit
   * @param data - Updated module fields
   * @returns The updated module
   */
  editModule: (
    moduleId: number,
    data: ModulePayload
  ): Promise<ApiResponse<Module>> =>
    apiFetch(`/modules/${moduleId}`, {
      method: "PUT",
      body: JSON.stringify(data),
    }),

  /**
   * Permanently delete a module by ID.
   * 
   * @param moduleId - ID of the module to delete
   * @returns Null response if deletion succeeds
   */
  deleteModule: (moduleId: number): Promise<ApiResponse<null>> =>
    apiFetch(`/modules/${moduleId}`, { method: "DELETE" }),

  /**
   * Fetch the current user's modules grouped by role.
   * 
   * @returns Modules the user is a student, tutor, or lecturer in
   */
  getMyModules: (): Promise<ApiResponse<MyModulesResponse>> =>
    apiFetch(`/modules/my`, { method: "GET" }),

  /**
   * Assign lecturers to a module.
   * 
   * @param moduleId - Target module ID
   * @param payload - List of user IDs to assign as lecturers
   */
  assignLecturers: (moduleId: number, payload: AssignLecturersRequest): Promise<ApiResponse<null>> =>
    apiFetch(`/modules/${moduleId}/lecturers`, {
      method: "POST",
      body: JSON.stringify(payload),
    }),

  /**
   * Remove lecturers from a module.
   * 
   * @param moduleId - Target module ID
   * @param payload - List of lecturer user IDs to remove
   */
  removeLecturers: (moduleId: number, payload: RemoveLecturersRequest): Promise<ApiResponse<null>> =>
    apiFetch(`/modules/${moduleId}/lecturers`, {
      method: "DELETE",
      body: JSON.stringify(payload),
    }),

  /**
   * Assign tutors to a module.
   * 
   * @param moduleId - Target module ID
   * @param payload - List of user IDs to assign as tutors
   */
  assignTutors: (moduleId: number, payload: AssignTutorsRequest): Promise<ApiResponse<null>> =>
    apiFetch(`/modules/${moduleId}/tutors`, {
      method: "POST",
      body: JSON.stringify(payload),
    }),

  /**
   * Remove tutors from a module.
   * 
   * @param moduleId - Target module ID
   * @param payload - List of tutor user IDs to remove
   */
  removeTutors: (moduleId: number, payload: RemoveTutorsRequest): Promise<ApiResponse<null>> =>
    apiFetch(`/modules/${moduleId}/tutors`, {
      method: "DELETE",
      body: JSON.stringify(payload),
    }),

  /**
   * Enroll students in a module.
   * 
   * @param moduleId - Target module ID
   * @param payload - List of user IDs to enroll
   */
  enrollStudents: (moduleId: number, payload: EnrollStudentsRequest): Promise<ApiResponse<null>> =>
    apiFetch(`/modules/${moduleId}/students`, {
      method: "POST",
      body: JSON.stringify(payload),
    }),

  /**
   * Remove students from a module.
   * 
   * @param moduleId - Target module ID
   * @param payload - List of student user IDs to remove
   */
  removeStudents: (moduleId: number, payload: RemoveStudentsRequest): Promise<ApiResponse<null>> =>
    apiFetch(`/modules/${moduleId}/students`, {
      method: "DELETE",
      body: JSON.stringify(payload),
    }),

  /**
   * Get a list of lecturers assigned to a module.
   * 
   * @param moduleId - ID of the module
   * @returns List of lecturer users
   */
  getLecturers: (moduleId: number): Promise<ApiResponse<{users: User[]}>> =>
    apiFetch(`/modules/${moduleId}/lecturers`, { method: "GET" }),

  /**
   * Get a list of tutors assigned to a module.
   * 
   * @param moduleId - ID of the module
   * @returns List of tutor users
   */
  getTutors: (moduleId: number): Promise<ApiResponse<{users: User[]}>> =>
    apiFetch(`/modules/${moduleId}/tutors`, { method: "GET" }),

  /**
   * Get a list of students enrolled in a module.
   * 
   * @param moduleId - ID of the module
   * @returns List of student users
   */
  getStudents: (moduleId: number): Promise<ApiResponse<{users: User[]}>> =>
    apiFetch(`/modules/${moduleId}/students`, { method: "GET" }),

  /**
   * Get all modules a user is involved in, with their role for each.
   * 
   * @param userId - ID of the user to query
   * @returns List of modules with user’s role in each
   */
  getModulesForUser: (userId: number): Promise<ApiResponse<UserModuleRole[]>> =>
    apiFetch(`/users/${userId}/modules`, { method: "GET" }),
};
