import type { LoginRequest, RegisterRequest, MeResponse, AuthUser } from '@/types/auth';
import { apiFetch } from '@/utils/api';
import type { ApiResponse } from '@/utils/api';

export const AuthService = {
  login: (data: LoginRequest): Promise<ApiResponse<AuthUser | null>> =>
    apiFetch('/auth/login', {
      method: 'POST',
      body: JSON.stringify(data),
    }),

  register: (data: RegisterRequest): Promise<ApiResponse<AuthUser | null>> =>
    apiFetch('/auth/register', {
      method: 'POST',
      body: JSON.stringify(data),
    }),

  me: (): Promise<ApiResponse<MeResponse>> =>
    apiFetch('/auth/me', {
      method: 'GET'
    })
};
