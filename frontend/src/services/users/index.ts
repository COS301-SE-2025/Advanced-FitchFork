import type {
  ListUsersRequest,
  ListUsersResponse,
  ModuleSummary,
  User,
  UserEditableFields,
} from "@/types/users";
import { apiFetch, type ApiResponse } from "@/utils/api";

/**
 * UsersService handles API requests related to user management.
 * All methods assume the current user has a valid admin JWT token.
 */
export const UsersService = {
  /**
   * Retrieves a paginated, filterable, searchable, and sortable list of users.
   */
  listUsers: (request: ListUsersRequest): Promise<ApiResponse<ListUsersResponse>> => {
    const params = new URLSearchParams();

    params.append("page", request.page.toString());
    params.append("per_page", request.per_page.toString());

    if (request.sort) {
      const sort = request.sort.map(({ field, order }) =>
        order === "descend" ? `-${field}` : field
      ).join(",");
      params.append("sort", sort);
    }

    if (request.query) params.append("query", request.query);
    if (request.email) params.append("email", request.email);
    if (request.student_number) params.append("student_number", request.student_number);
    if (typeof request.admin === "boolean") params.append("admin", request.admin.toString());

    return apiFetch(`/users?${params.toString()}`, { method: "GET" });
  },

  /**
   * Retrieves details of a single user by ID.
   */
  getUser: (userId: number): Promise<ApiResponse<User>> =>
    apiFetch(`/users/${userId}`, { method: "GET" }),

  /**
   * Updates a user's details.
   */
  editUser: (userId: number, user: User): Promise<ApiResponse<User>> => {
    const payload: UserEditableFields = {
      student_number: user.student_number,
      email: user.email,
      admin: user.admin,
    };

    return apiFetch(`/users/${userId}`, {
      method: "PUT",
      body: JSON.stringify(payload),
    });
  },

  /**
   * Deletes a user by ID.
   */
  deleteUser: (userId: number): Promise<ApiResponse<null>> =>
    apiFetch(`/users/${userId}`, { method: "DELETE" }),

  /**
   * Returns the list of modules a user is involved in.
   */
  getUserModules: (userId: number): Promise<ApiResponse<ModuleSummary[]>> =>
    apiFetch(`/users/${userId}/modules`, { method: "GET" }),
};
