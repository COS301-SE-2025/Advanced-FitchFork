import { API_BASE_URL } from '@/config/api';
import { getAuthenticatedUser } from '@/services/auth';
import { getMyModules } from '@/services/modules';
import type { Module, ModuleRole } from '@/types/modules';
import type { User } from '@/types/users';

export interface UserModuleRole extends User {
  modules: {
    Lecturer: Module[];
    Tutor: Module[];
    Student: Module[];
    Flat: (Module & { role: ModuleRole })[];
  };
  profilePictureUrl: string;
}

export const loadAuthSession = async (stored: any): Promise<UserModuleRole | null> => {
  const meRes = await getAuthenticatedUser();
  if (!meRes.success || !meRes.data) return null;

  const userData = meRes.data;
  const modRes = await getMyModules();
  if (!modRes.success || !modRes.data) return null;

  const grouped = modRes.data;

  const flat = [
    ...grouped.as_lecturer.map((m) => ({ ...m, role: 'Lecturer' as const })),
    ...grouped.as_tutor.map((m) => ({ ...m, role: 'Tutor' as const })),
    ...grouped.as_student.map((m) => ({ ...m, role: 'Student' as const })),
  ];

  const result: UserModuleRole = {
    ...userData,
    profilePictureUrl: `${API_BASE_URL}/auth/avatar/${userData.id}?bust=${Date.now()}`,
    modules: {
      Lecturer: grouped.as_lecturer,
      Tutor: grouped.as_tutor,
      Student: grouped.as_student,
      Flat: flat,
    },
  };

  localStorage.setItem(
    'auth',
    JSON.stringify({
      ...stored,
      user: userData,
      modules: flat,
    }),
  );

  return result;
};
