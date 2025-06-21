import type { LoginRequest, RegisterRequest, MeResponse, AuthUser } from '@/types/auth';
import type { ModuleRole } from '@/types/modules';
import { apiFetch, apiUpload, apiDownload } from '@/utils/api';
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
      method: 'GET',
    }),

  requestPasswordReset: (email: string): Promise<ApiResponse<null>> =>
    apiFetch('/auth/request-password-reset', {
      method: 'POST',
      body: JSON.stringify({ email }),
    }),

  resetPassword: (token: string, newPassword: string): Promise<ApiResponse<null>> =>
    apiFetch('/auth/reset-password', {
      method: 'POST',
      body: JSON.stringify({
        token,
        new_password: newPassword,
      }),
    }),

  verifyResetToken: (token: string): Promise<ApiResponse<{ email_hint?: string }>> =>
    apiFetch('/auth/verify-reset-token', {
      method: 'POST',
      body: JSON.stringify({ token }),
    }),

  hasRole: (
    module_id: number,
    role: ModuleRole
  ): Promise<ApiResponse<{ has_role: boolean }>> =>
    apiFetch(`/auth/has-role?module_id=${module_id}&role=${role}`, {
      method: 'GET',
    }),

  uploadProfilePicture: (form: FormData): Promise<ApiResponse<{ profile_picture_path: string }>> =>
    apiUpload('/auth/upload-profile-picture', form),

  downloadProfilePicture: (): Promise<void> =>
    apiDownload('/auth/avatar/me'),

//   getProfilePictureBlobUrl: async (): Promise<string | null> => {
//   const url = `${API_BASE_URL}/auth/avatar/me`;

//   const stored = localStorage.getItem('auth');
//   let token: string | null = null;

//   if (stored) {
//     try {
//       const parsed = JSON.parse(stored);
//       token = parsed?.token || null;
//     } catch {
//       token = null;
//     }
//   }

//   const headers: HeadersInit = {};
//   if (token) headers['Authorization'] = `Bearer ${token}`;

//   try {
//     const res = await fetch(url, { method: 'GET', headers });
//     if (!res.ok) return null;

//     const blob = await res.blob();
//     return URL.createObjectURL(blob);
//   } catch {
//     return null;
//   }
// }
};
