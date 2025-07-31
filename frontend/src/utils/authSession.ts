import { API_BASE_URL } from '@/config/api';
import { getAuthenticatedUser } from '@/services/auth';
import { getMyModules } from '@/services/modules';
import type { Module, ModuleRole } from '@/types/modules';
import type { User } from '@/types/users';

export interface UserModuleRole extends User {
  modules: {
    lecturer: Module[];
    assistant_lecturer: Module[];
    tutor: Module[];
    student: Module[];
    flat: (Module & { role: ModuleRole })[];
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
    ...grouped.as_lecturer.map((m) => ({ ...m, role: 'lecturer' as const })),
    ...grouped.as_assistant_lecturer.map((m) => ({ ...m, role: 'assistant_lecturer' as const })),
    ...grouped.as_tutor.map((m) => ({ ...m, role: 'tutor' as const })),
    ...grouped.as_student.map((m) => ({ ...m, role: 'student' as const })),
  ];

  const result: UserModuleRole = {
    ...userData,
    profilePictureUrl: `${API_BASE_URL}/auth/avatar/${userData.id}?bust=${Date.now()}`,
    modules: {
      lecturer: grouped.as_lecturer,
      assistant_lecturer: grouped.as_assistant_lecturer,
      tutor: grouped.as_tutor,
      student: grouped.as_student,
      flat,
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
