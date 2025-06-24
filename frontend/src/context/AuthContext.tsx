import type { AuthUser, PostLoginResponse, PostRegisterResponse } from '@/types/auth';
import type { User } from '@/types/users';
import type { Module, ModuleRole } from '@/types/modules';
import React, { createContext, useContext, useState, useEffect } from 'react';
import type { ApiResponse } from '@/types/common';
import { API_BASE_URL } from '@/utils/api';
import { getMyModules } from '@/services/modules';
import {
  getAuthenticatedUser,
  login as loginService,
  register as registerService,
} from '@/services/auth';

interface UserModuleRole extends Module {
  role: ModuleRole;
}

interface ModulesByRole {
  Lecturer: Module[];
  Tutor: Module[];
  Student: Module[];
}

interface AuthContextType {
  user: User | null;
  modules: UserModuleRole[];
  modulesByRole: ModulesByRole;
  loading: boolean;
  profilePictureUrl: string | null;
  setProfilePictureUrl: (url: string | null) => void;

  // core actions
  login: (student_number: string, password: string) => Promise<PostLoginResponse>;
  register: (
    student_number: string,
    email: string,
    password: string,
  ) => Promise<PostRegisterResponse>;
  logout: () => void;
  isExpired: () => boolean;

  // derived info
  isAdmin: boolean;
  isUser: boolean;
  getModuleRole: (moduleId: number) => ModuleRole | null;
  hasModuleRole: (moduleId: number, role: ModuleRole) => boolean;
  isLecturer: (moduleId: number) => boolean;
  isTutor: (moduleId: number) => boolean;
  isStudent: (moduleId: number) => boolean;
}

const AuthContext = createContext<AuthContextType | undefined>(undefined);

export const AuthProvider: React.FC<{ children: React.ReactNode }> = ({ children }) => {
  const [user, setUser] = useState<User | null>(null);
  const [modules, setModules] = useState<UserModuleRole[]>([]);
  const [modulesByRole, setModulesByRole] = useState<ModulesByRole>({
    Lecturer: [],
    Tutor: [],
    Student: [],
  });
  const [loading, setLoading] = useState(true);
  const [profilePictureUrl, setProfilePictureUrl] = useState<string | null>(null);

  useEffect(() => {
    const loadUser = async () => {
      const stored = localStorage.getItem('auth');
      if (stored) {
        try {
          const parsed = JSON.parse(stored);
          if (isExpired()) {
            logout();
            return;
          }

          const meRes = await getAuthenticatedUser();
          if (meRes.success && meRes.data) {
            const userData = meRes.data;
            setUser(userData);
            setProfilePictureUrl(`${API_BASE_URL}/auth/avatar/${userData.id}?bust=${Date.now()}`);

            const modRes = await getMyModules();
            if (modRes.success && modRes.data) {
              const grouped = modRes.data;

              const flat: UserModuleRole[] = [
                ...grouped.as_lecturer.map((m) => ({ ...m, role: 'Lecturer' as const })),
                ...grouped.as_tutor.map((m) => ({ ...m, role: 'Tutor' as const })),
                ...grouped.as_student.map((m) => ({ ...m, role: 'Student' as const })),
              ];

              setModules(flat);
              setModulesByRole({
                Lecturer: grouped.as_lecturer,
                Tutor: grouped.as_tutor,
                Student: grouped.as_student,
              });

              localStorage.setItem(
                'auth',
                JSON.stringify({
                  ...parsed,
                  user: userData,
                  modules: flat,
                }),
              );
            } else {
              logout();
            }
          } else {
            logout();
          }
        } catch {
          logout();
        }
      }
      setLoading(false);
    };

    loadUser();
  }, []);

  const login = async (
    student_number: string,
    password: string,
  ): Promise<ApiResponse<AuthUser | null>> => {
    try {
      const res = await loginService(student_number, password);
      if (res.success && res.data) {
        const { token, expires_at, ...user } = res.data;

        localStorage.setItem(
          'auth',
          JSON.stringify({
            user,
            token,
            expires_at,
            modules: [],
          }),
        );

        const meRes = await getAuthenticatedUser();
        if (meRes.success && meRes.data) {
          const userData = meRes.data;
          setUser(userData);
          setProfilePictureUrl(`${API_BASE_URL}/auth/avatar/${userData.id}?bust=${Date.now()}`);

          const modRes = await getMyModules();
          if (modRes.success && modRes.data) {
            const grouped = modRes.data;

            const flat: UserModuleRole[] = [
              ...grouped.as_lecturer.map((m) => ({ ...m, role: 'Lecturer' as const })),
              ...grouped.as_tutor.map((m) => ({ ...m, role: 'Tutor' as const })),
              ...grouped.as_student.map((m) => ({ ...m, role: 'Student' as const })),
            ];

            setModules(flat);
            setModulesByRole({
              Lecturer: grouped.as_lecturer,
              Tutor: grouped.as_tutor,
              Student: grouped.as_student,
            });

            localStorage.setItem(
              'auth',
              JSON.stringify({
                user: userData,
                token,
                expires_at,
                modules: flat,
              }),
            );
          }
        }

        return res;
      }

      return res;
    } catch (err: any) {
      return {
        success: false,
        data: null,
        message: err.message || 'Unexpected error during login',
      };
    }
  };

  const register = async (
    student_number: string,
    email: string,
    password: string,
  ): Promise<ApiResponse<AuthUser | null>> => {
    try {
      return await registerService(student_number, email, password);
    } catch (err: any) {
      return {
        success: false,
        data: null,
        message: err.message || 'Unexpected error during registration',
      };
    }
  };

  const logout = () => {
    localStorage.removeItem('auth');
    setUser(null);
    setModules([]);
    setModulesByRole({ Lecturer: [], Tutor: [], Student: [] });
    window.location.href = '/login';
  };

  const isExpired = (): boolean => {
    const stored = localStorage.getItem('auth');
    if (!stored) return true;
    try {
      const { expires_at } = JSON.parse(stored);
      return !expires_at || new Date(expires_at) < new Date();
    } catch {
      return true;
    }
  };

  const isAdmin = !!user?.admin;
  const isUser = !!user && !user.admin;

  const getModuleRole = (moduleId: number): ModuleRole | null => {
    const mod = modules.find((m) => m.id === moduleId);
    return mod?.role || null;
  };

  const hasModuleRole = (moduleId: number, role: ModuleRole): boolean => {
    return getModuleRole(moduleId) === role;
  };

  const isLecturer = (moduleId: number): boolean => getModuleRole(moduleId) === 'Lecturer';
  const isTutor = (moduleId: number): boolean => getModuleRole(moduleId) === 'Tutor';
  const isStudent = (moduleId: number): boolean => getModuleRole(moduleId) === 'Student';

  return (
    <AuthContext.Provider
      value={{
        user,
        modules,
        modulesByRole,
        loading,
        profilePictureUrl,
        setProfilePictureUrl,
        login,
        register,
        logout,
        isExpired,
        isAdmin,
        isUser,
        getModuleRole,
        hasModuleRole,
        isLecturer,
        isTutor,
        isStudent,
      }}
    >
      {children}
    </AuthContext.Provider>
  );
};

export const useAuth = (): AuthContextType => {
  const ctx = useContext(AuthContext);
  if (!ctx) throw new Error('useAuth must be used within AuthProvider');
  return ctx;
};

export default AuthProvider;
