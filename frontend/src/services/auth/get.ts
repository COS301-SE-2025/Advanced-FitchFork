import type { GetAuthenticatedUserResponse, GetHasRoleResponse } from "@/types/auth";
import type { ModuleRole } from "@/types/modules";
import { apiDownload, apiFetch } from "@/utils/api";

export const getAuthenticatedUser = async (): Promise<GetAuthenticatedUserResponse> => {
  return apiFetch('/auth/me');
};

export const hasRole = async (
  moduleId: number,
  role: ModuleRole
): Promise<GetHasRoleResponse> => {
  return apiFetch(`/auth/has-role?module_id=${moduleId}&role=${role}`);
};

export const downloadProfilePicture = async (): Promise<void> => {
  return apiDownload('/auth/avatar/me');
};

export const getProfilePictureBlobUrl = async (): Promise<string | null> => {
  const url = `${import.meta.env.VITE_API_BASE_URL || 'http://localhost:3000/api'}/auth/avatar/me`;

  const stored = localStorage.getItem('auth');
  let token: string | null = null;

  if (stored) {
    try {
      const parsed = JSON.parse(stored);
      token = parsed?.token || null;
    } catch {
      token = null;
    }
  }

  const headers: HeadersInit = {};
  if (token) headers['Authorization'] = `Bearer ${token}`;

  try {
    const res = await fetch(url, { method: 'GET', headers });
    if (!res.ok) return null;

    const blob = await res.blob();
    return URL.createObjectURL(blob);
  } catch {
    return null;
  }
};