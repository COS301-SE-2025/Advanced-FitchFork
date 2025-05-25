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

// Helper to simulate delay
function delay(ms = 200 + Math.random() * 400): Promise<void> {
  return new Promise(resolve => setTimeout(resolve, ms));
}

export const UsersService = {
  listUsers: async (_: ListUsersRequest): Promise<ApiResponse<ListUsersResponse>> => {
    await delay();
    return {
      success: true,
      data: {
        users: mockUsers,
        page: 1,
        per_page: mockUsers.length,
        total: mockUsers.length,
      },
      message: "Mocked user list (ignoring filters and sorting)",
    };
  },

  editUser: async (userId: number, user: User): Promise<ApiResponse<User>> => {
    await delay();

    const index = mockUsers.findIndex(u => u.id === userId);
    if (index === -1) {
      return {
        success: false,
        message: `User with id ${userId} not found`,
        data: undefined as any,
      };
    }

    const updatedUser: User = {
      ...mockUsers[index],
      student_number: user.student_number,
      email: user.email,
      admin: user.admin,
      updated_at: new Date().toISOString(),
    };

    mockUsers[index] = updatedUser;

    return {
      success: true,
      data: updatedUser,
      message: "Mocked user updated successfully",
    };
  },

  deleteUser: async (_: number): Promise<ApiResponse<null>> => {
    await delay();
    mockUsers.pop();
    return {
      success: true,
      data: null,
      message: "Mocked user deleted (last one removed)",
    };
  },

  getModulesForUser: async (_: number): Promise<ApiResponse<UserModulesResponse>> => {
    await delay();
    return {
      success: true,
      data: {
        modules: mockModules,
      },
      message: "Mocked modules for any user",
    };
  },
};
