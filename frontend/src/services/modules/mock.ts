import type {
  Module,
  ModulePayload,
  ListModulesRequest,
  ListModulesResponse,
  ModuleDetailsResponse,
  MyModulesResponse,
  ListLecturersResponse,
  ListTutorsResponse,
  ListStudentsResponse,
  UserModuleRole,
  EnrollStudentsRequest,
} from "@/types/modules";
import type { User } from "@/types/users";
import type { ApiResponse } from "@/utils/api";

const now = new Date().toISOString();

function delay(ms = 200 + Math.random() * 400) {
  return new Promise(resolve => setTimeout(resolve, ms));
}

const mockUsers: User[] = [
  { id: 1, student_number: "20230001", email: "alice@up.ac.za", admin: true, created_at: now, updated_at: now },
  { id: 2, student_number: "20230002", email: "bob@up.ac.za", admin: false, created_at: now, updated_at: now },
  { id: 3, student_number: "20230003", email: "carol@up.ac.za", admin: false, created_at: now, updated_at: now },
  { id: 4, student_number: "20230004", email: "daniel@up.ac.za", admin: false, created_at: now, updated_at: now },
  { id: 5, student_number: "20230005", email: "eve@up.ac.za", admin: true, created_at: now, updated_at: now },
  { id: 6, student_number: "20230006", email: "frank@up.ac.za", admin: false, created_at: now, updated_at: now },
];

const mockModules: Module[] = [
  { id: 101, code: "COS332", year: 2025, description: "Networks and Security", credits: 16, created_at: now, updated_at: now },
  { id: 102, code: "COS344", year: 2025, description: "Computer Graphics", credits: 16, created_at: now, updated_at: now },
  { id: 103, code: "COS333", year: 2025, description: "Programming Languages", credits: 16, created_at: now, updated_at: now },
  { id: 104, code: "INF214", year: 2025, description: "Information Systems", credits: 12, created_at: now, updated_at: now },
  { id: 105, code: "MIT301", year: 2024, description: "Advanced Software Engineering", credits: 24, created_at: now, updated_at: now },
  { id: 106, code: "PHY101", year: 2023, description: "Introductory Physics", credits: 12, created_at: now, updated_at: now },
  { id: 107, code: "STK120", year: 2024, description: "Statistics", credits: 8, created_at: now, updated_at: now },
  { id: 108, code: "ECS150", year: 2023, description: "Microeconomics", credits: 8, created_at: now, updated_at: now },
  { id: 109, code: "WTW285", year: 2025, description: "Differential Equations", credits: 16, created_at: now, updated_at: now },
  { id: 110, code: "COS730", year: 2025, description: "Machine Learning", credits: 32, created_at: now, updated_at: now },
];

export const ModulesService = {
  listModules: async (_: ListModulesRequest): Promise<ApiResponse<ListModulesResponse>> => {
    await delay();
    return {
      success: true,
      data: {
        modules: mockModules,
        page: 1,
        per_page: mockModules.length,
        total: mockModules.length,
      },
      message: "Mocked module list",
    };
  },

  getModuleDetails: async (_: number): Promise<ApiResponse<ModuleDetailsResponse>> => {
    await delay();
    return {
      success: true,
      data: {
        ...mockModules[0],
        lecturers: [mockUsers[0]],
        tutors: [mockUsers[2]],
        students: [mockUsers[1], mockUsers[3]],
      },
      message: "Mocked module details (first module)",
    };
  },

  createModule: async (_: ModulePayload): Promise<ApiResponse<Module>> => {
    await delay();
    const newModule: Module = {
      id: Math.floor(Math.random() * 1000 + 200),
      code: "NEW123",
      year: 2025,
      description: "Mock Created Module",
      credits: 10,
      created_at: now,
      updated_at: now,
    };
    mockModules.push(newModule);
    return {
      success: true,
      data: newModule,
      message: "Mocked module created",
    };
  },

  editModule: async (_: number, __: ModulePayload): Promise<ApiResponse<Module>> => {
    await delay();
    const edited = { ...mockModules[0], description: "Edited Description", updated_at: now };
    mockModules[0] = edited;
    return {
      success: true,
      data: edited,
      message: "Mocked module edited",
    };
  },

  deleteModule: async (_: number): Promise<ApiResponse<null>> => {
    await delay();
    return {
      success: true,
      data: null,
      message: "Mocked module deleted",
    };
  },

  getMyModules: async (): Promise<ApiResponse<MyModulesResponse>> => {
    await delay();
    return {
      success: true,
      data: {
        as_student: [mockModules[2], mockModules[3]],
        as_tutor: [mockModules[4]],
        as_lecturer: [mockModules[0]],
      },
      message: "Mocked my modules",
    };
  },

  assignLecturers: async (): Promise<ApiResponse<null>> => {
    await delay();
    return {
      success: true,
      data: null,
      message: "Mocked assign lecturers",
    };
  },

  removeLecturers: async (): Promise<ApiResponse<null>> => {
    await delay();
    return {
      success: true,
      data: null,
      message: "Mocked remove lecturers",
    };
  },

  assignTutors: async (): Promise<ApiResponse<null>> => {
    await delay();
    return {
      success: true,
      data: null,
      message: "Mocked assign tutors",
    };
  },

  removeTutors: async (): Promise<ApiResponse<null>> => {
    await delay();
    return {
      success: true,
      data: null,
      message: "Mocked remove tutors",
    };
  },

  enrollStudents: async (_moduleId: number, _payload : EnrollStudentsRequest): Promise<ApiResponse<null>> => {
    await delay();
    return {
      success: true,
      data: null,
      message: "Mocked enroll students",
    };
  },

  removeStudents: async (): Promise<ApiResponse<null>> => {
    await delay();
    return {
      success: true,
      data: null,
      message: "Mocked remove students",
    };
  },

  getLecturers: async (_: number): Promise<ApiResponse<{users: User[]}>> => {
    await delay();
    return {
      success: true,
      data: { users: [mockUsers[0]] },
      message: "Mocked lecturers list",
    };
  },

  getTutors: async (_: number): Promise<ApiResponse<{users: User[]}>> => {
    await delay();
    return {
      success: true,
      data: { users: [mockUsers[2]] },
      message: "Mocked tutors list",
    };
  },

  getStudents: async (_: number): Promise<ApiResponse<{users: User[]}>> => {
    await delay();
    return {
      success: true,
      data: { users: [mockUsers[1], mockUsers[3]] },
      message: "Mocked students list",
    };
  },

  getModulesForUser: async (): Promise<ApiResponse<UserModuleRole[]>> => {
    await delay();
    return {
      success: true,
      data: mockModules.map((m, i) => ({
        ...m,
        role: i % 3 === 0 ? "Lecturer" : i % 3 === 1 ? "Tutor" : "Student",
      })),
      message: "Mocked modules for user",
    };
  },
};
