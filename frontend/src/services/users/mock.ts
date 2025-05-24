import type {
  ListUsersRequest,
  ListUsersResponse,
  UserEditableFields,
  UserModulesResponse,
} from "@/types/users";
import type { ApiResponse } from "@/utils/api";
import type { User } from "@/types/users";
import type { ModuleSummary } from "@/types/users";

const now = new Date().toISOString();

let mockUsers: User[] = [
  { id: 1, student_number: "20230001", email: "alice@up.ac.za", admin: true, created_at: now, updated_at: now },
  { id: 2, student_number: "20230002", email: "bob@up.ac.za", admin: false, created_at: now, updated_at: now },
  { id: 3, student_number: "20230003", email: "carol@up.ac.za", admin: false, created_at: now, updated_at: now },
  { id: 4, student_number: "20230004", email: "daniel@up.ac.za", admin: false, created_at: now, updated_at: now },
  { id: 5, student_number: "20230005", email: "eve@up.ac.za", admin: true, created_at: now, updated_at: now },
  { id: 6, student_number: "20230006", email: "frank@up.ac.za", admin: false, created_at: now, updated_at: now },
];

const mockModules: ModuleSummary[] = [
  {
    id: 101,
    code: "COS332",
    year: 2025,
    description: "Networks and Security",
    credits: 16,
    role: "Student",
    created_at: now,
    updated_at: now,
  },
  {
    id: 102,
    code: "COS344",
    year: 2025,
    description: "Computer Graphics",
    credits: 16,
    role: "Lecturer",
    created_at: now,
    updated_at: now,
  },
  {
    id: 103,
    code: "COS333",
    year: 2024,
    description: "Programming Languages",
    credits: 16,
    role: "Tutor",
    created_at: now,
    updated_at: now,
  },
];

export const UsersService = {
  listUsers: async (request: ListUsersRequest): Promise<ApiResponse<ListUsersResponse>> => {
    let results = [...mockUsers];

    if (request.query) {
      const query = request.query.toLowerCase();
      results = results.filter(
        u =>
          u.email.toLowerCase().includes(query) ||
          u.student_number.includes(query)
      );
    }

    if (request.email) {
      results = results.filter(u => u.email === request.email);
    }

    if (request.student_number) {
      results = results.filter(u => u.student_number === request.student_number);
    }

    if (typeof request.admin === "boolean") {
      results = results.filter(u => u.admin === request.admin);
    }

    if (request.sort) {
      for (const { field, order } of [...request.sort].reverse()) {
        results.sort((a, b) => {
          const aVal = (a as any)[field];
          const bVal = (b as any)[field];
          if (aVal < bVal) return order === "asc" ? -1 : 1;
          if (aVal > bVal) return order === "asc" ? 1 : -1;
          return 0;
        });
      }
    }

    const page = request.page;
    const perPage = request.per_page;
    const start = (page - 1) * perPage;
    const paginated = results.slice(start, start + perPage);

    return {
      success: true,
      data: {
        users: paginated,
        page,
        per_page: perPage,
        total: results.length,
      },
      message: "Mocked user list",
    };
  },

  editUser: async (userId: number, payload: UserEditableFields): Promise<ApiResponse<UserEditableFields>> => {
    const userIndex = mockUsers.findIndex(u => u.id === userId);
    if (userIndex === -1) throw new Error("User not found");
    mockUsers[userIndex] = {
      ...mockUsers[userIndex],
      ...payload,
      updated_at: now,
    };
    return {
      success: true,
      data: payload,
      message: "Mocked user updated",
    };
  },

  deleteUser: async (userId: number): Promise<ApiResponse<null>> => {
    mockUsers = mockUsers.filter(u => u.id !== userId);
    return {
      success: true,
      data: null,
      message: "Mocked user deleted",
    };
  },

  getModulesForUser: async (_userId: number): Promise<ApiResponse<UserModulesResponse>> => {
    return {
      success: true,
      data: { modules: mockModules },
      message: "Mocked user modules",
    };
  },
};
