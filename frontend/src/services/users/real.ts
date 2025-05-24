import type {
  ListUsersRequest,
  ListUsersResponse,
  UserEditableFields,
  UserModulesResponse,
} from "@/types/users";
import { apiFetch, type ApiResponse} from "@/utils/api";

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
        order === "desc" ? `-${field}` : field
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
   * Updates a user's details.
   */
  editUser: (userId: number, payload: UserEditableFields): Promise<ApiResponse<UserEditableFields>> =>
    apiFetch(`/users/${userId}`, {
      method: "PUT",
      body: JSON.stringify(payload),
    }),


  /**
   * Deletes a user by ID.
   */
  deleteUser: (userId: number): Promise<ApiResponse<null>> =>
    apiFetch(`/users/${userId}`, { method: "DELETE" }),

  /**
   * Returns the list of modules a user is involved in.
   */
  getModulesForUser: (userId: number): Promise<ApiResponse<UserModulesResponse>> =>
    apiFetch(`/users/${userId}/modules`, { method: "GET" }),
};
