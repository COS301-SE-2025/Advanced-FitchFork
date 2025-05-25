import type {
  LoginRequest,
  RegisterRequest,
  MeResponse,
  AuthUser,
  UserModule,
} from '@/types/auth';
import type { ApiResponse } from '@/utils/api';

const now = new Date().toISOString();
const expires = new Date(Date.now() + 60 * 60 * 1000).toISOString(); // 1 hour from now

function delay(ms = 200 + Math.random() * 400) {
  return new Promise(resolve => setTimeout(resolve, ms));
}

const mockModules: UserModule[] = [
  {
    id: 1,
    code: 'COS332',
    year: 2025,
    description: 'Networks and Security',
    credits: 16,
    role: 'Student',
    created_at: now,
    updated_at: now,
  },
  {
    id: 2,
    code: 'COS344',
    year: 2025,
    description: 'Computer Graphics',
    credits: 16,
    role: 'Student',
    created_at: now,
    updated_at: now,
  },
  {
    id: 3,
    code: 'COS333',
    year: 2025,
    description: 'Programming Languages',
    credits: 16,
    role: 'Student',
    created_at: now,
    updated_at: now,
  },
  {
    id: 4,
    code: 'INF214',
    year: 2025,
    description: 'Information Systems',
    credits: 12,
    role: 'Student',
    created_at: now,
    updated_at: now,
  },
];

const baseUser = {
  id: 1,
  student_number: 'u23571561',
  email: 'u23571561@tuks.co.za',
  admin: true,
  created_at: now,
  updated_at: now,
};

const mockAuthUser: AuthUser = {
  ...baseUser,
  token: 'mock-token',
  expires_at: expires,
};

const mockMe: MeResponse = {
  ...baseUser,
  modules: mockModules,
};

export const AuthService = {
  login: async (_: LoginRequest): Promise<ApiResponse<AuthUser | null>> => {
    await delay();
    return {
      success: true,
      data: mockAuthUser,
      message: 'Logged in (mock)',
    };
  },

  register: async (_: RegisterRequest): Promise<ApiResponse<AuthUser | null>> => {
    await delay();
    return {
      success: true,
      data: mockAuthUser,
      message: 'Registered (mock)',
    };
  },

  me: async (): Promise<ApiResponse<MeResponse>> => {
    await delay();
    return {
      success: true,
      data: mockMe,
      message: 'Fetched user data (mock)',
    };
  },
};
