import type { AuthUser, PostLoginResponse, PostRegisterResponse } from '@/types/auth';
import type { User } from '@/types/users';
import type { Module, ModuleRole } from '@/types/modules';
import React, { createContext, useContext, useState, useEffect } from 'react';
import type { ApiResponse } from '@/types/common';
import { login as loginService, register as registerService } from '@/services/auth';
import { loadAuthSession } from '@/utils/authSession';

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
  login: (username: string, password: string) => Promise<PostLoginResponse>;
  register: (username: string, email: string, password: string) => Promise<PostRegisterResponse>;
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
          if (isExpired()) return logout();

          const session = await loadAuthSession(parsed);
          if (!session) return logout();

          setUser(session);
          setProfilePictureUrl(session.profilePictureUrl);
          setModules(session.modules.Flat);
          setModulesByRole({
            Lecturer: session.modules.Lecturer,
            Tutor: session.modules.Tutor,
            Student: session.modules.Student,
          });
        } catch {
          logout();
        }
      }
      setLoading(false);
    };

    loadUser();
  }, []);

  const login = async (
    username: string,
    password: string,
  ): Promise<ApiResponse<AuthUser | null>> => {
    try {
      const res = await loginService(username, password);
      if (!res.success || !res.data) return res;

      const { token, expires_at, ...user } = res.data;
      localStorage.setItem('auth', JSON.stringify({ user, token, expires_at, modules: [] }));

      const session = await loadAuthSession({ token, expires_at });
      if (session) {
        setUser(session);
        setProfilePictureUrl(session.profilePictureUrl);
        setModules(session.modules.Flat);
        setModulesByRole({
          Lecturer: session.modules.Lecturer,
          Tutor: session.modules.Tutor,
          Student: session.modules.Student,
        });
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
    username: string,
    email: string,
    password: string,
  ): Promise<ApiResponse<AuthUser | null>> => {
    try {
      return await registerService(username, email, password);
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
