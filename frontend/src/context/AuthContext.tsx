import type { PostLoginResponse, PostRegisterResponse } from '@/types/auth';
import type { User } from '@/types/users';
import type { Module, ModuleRole } from '@/types/modules';
import React, { createContext, useContext, useState, useEffect } from 'react';
import { login as loginService, register as registerService } from '@/services/auth';
import { loadAuthSession } from '@/utils/authSession';

interface UserModuleRole extends Module {
  role: ModuleRole;
}

interface ModulesByRole {
  lecturer: Module[];
  assistant_lecturer: Module[];
  tutor: Module[];
  student: Module[];
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
  token: string | null;

  // derived info
  isAdmin: boolean;
  isUser: boolean;
  getModuleRole: (moduleId: number) => ModuleRole | null;

  isLecturer: (moduleId: number) => boolean;
  isAssistantLecturer: (moduleId: number) => boolean;
  isTutor: (moduleId: number) => boolean;
  isStudent: (moduleId: number) => boolean;
  isStaff: (moduleId: number) => boolean;

  hasModuleRole: (role: ModuleRole) => boolean;
  hasLecturerRole: () => boolean;
  hasAssistantLecturerRole: () => boolean;
  hasTutorRole: () => boolean;
  hasStudentRole: () => boolean;
  hasStaffRole: () => boolean;

  print: () => void;
}

const AuthContext = createContext<AuthContextType | undefined>(undefined);

type StoredAuth = {
  user?: Partial<User>;
  token?: string;
  expires_at?: string;
  modules?: unknown;
};

const readStoredAuth = (): StoredAuth | null => {
  const raw = localStorage.getItem('auth');
  if (!raw) return null;
  try {
    return JSON.parse(raw) as StoredAuth;
  } catch {
    return null;
  }
};

export const AuthProvider: React.FC<{ children: React.ReactNode }> = ({ children }) => {
  const [user, setUser] = useState<User | null>(null);
  const [modules, setModules] = useState<UserModuleRole[]>([]);
  const [modulesByRole, setModulesByRole] = useState<ModulesByRole>({
    lecturer: [],
    assistant_lecturer: [],
    tutor: [],
    student: [],
  });
  const [loading, setLoading] = useState(true);
  const [profilePictureUrl, setProfilePictureUrl] = useState<string | null>(null);
  const [token, setToken] = useState<string | null>(null);

  useEffect(() => {
    const loadUser = async () => {
      const stored = readStoredAuth();
      if (stored?.token) {
        setToken(stored.token);
        try {
          if (isExpired()) return logout();

          const session = await loadAuthSession({
            token: stored.token,
            expires_at: stored.expires_at,
          });
          if (!session) return logout();

          setUser(session);
          setProfilePictureUrl(session.profilePictureUrl);
          setModules(session.modules.flat);
          setModulesByRole({
            lecturer: session.modules.lecturer,
            assistant_lecturer: session.modules.assistant_lecturer,
            tutor: session.modules.tutor,
            student: session.modules.student,
          });
        } catch {
          logout();
        }
      }
      setLoading(false);
    };

    loadUser();
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);

  const login = async (username: string, password: string): Promise<PostLoginResponse> => {
    try {
      const res = await loginService(username, password); // assumed to return PostLoginResponse
      if (!res.success || !res.data) return res;

      const { token: newToken, expires_at, ...user } = res.data;
      localStorage.setItem(
        'auth',
        JSON.stringify({ user, token: newToken, expires_at, modules: [] }),
      );
      setToken(newToken);

      const session = await loadAuthSession({ token: newToken, expires_at });
      if (session) {
        setUser(session);
        setProfilePictureUrl(session.profilePictureUrl);
        setModules(session.modules.flat);
        setModulesByRole({
          lecturer: session.modules.lecturer,
          assistant_lecturer: session.modules.assistant_lecturer,
          tutor: session.modules.tutor,
          student: session.modules.student,
        });
      }

      return res;
    } catch (err: any) {
      return {
        success: false,
        data: null,
        message: err?.message || 'Unexpected error during login',
      } as PostLoginResponse;
    }
  };

  const register = async (
    username: string,
    email: string,
    password: string,
  ): Promise<PostRegisterResponse> => {
    try {
      return await registerService(username, email, password);
    } catch (err: any) {
      return {
        success: false,
        data: null,
        message: err?.message || 'Unexpected error during registration',
      } as PostRegisterResponse;
    }
  };

  const logout = () => {
    localStorage.removeItem('auth');
    setToken(null);
    setUser(null);
    setModules([]);
    setModulesByRole({ lecturer: [], assistant_lecturer: [], tutor: [], student: [] });
    window.location.href = '/login';
  };

  const isExpired = (): boolean => {
    const stored = readStoredAuth();
    if (!stored?.expires_at) return true;
    try {
      return new Date(stored.expires_at) < new Date();
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

  const isLecturer = (moduleId: number): boolean => getModuleRole(moduleId) === 'lecturer';
  const isAssistantLecturer = (moduleId: number): boolean =>
    getModuleRole(moduleId) === 'assistant_lecturer';
  const isTutor = (moduleId: number): boolean => getModuleRole(moduleId) === 'tutor';
  const isStudent = (moduleId: number): boolean => getModuleRole(moduleId) === 'student';
  const isStaff = (moduleId: number): boolean =>
    isTutor(moduleId) || isAssistantLecturer(moduleId) || isLecturer(moduleId) || isAdmin;

  const hasModuleRole = (role: ModuleRole): boolean => modules.some((m) => m.role === role);

  const hasLecturerRole = (): boolean => hasModuleRole('lecturer');
  const hasAssistantLecturerRole = (): boolean => hasModuleRole('assistant_lecturer');
  const hasTutorRole = (): boolean => hasModuleRole('tutor');
  const hasStudentRole = (): boolean => hasModuleRole('student');
  const hasStaffRole = (): boolean =>
    hasLecturerRole() || hasAssistantLecturerRole() || hasTutorRole();

  const print = () => {
    console.group('%c[AuthContext State]', 'color: #4CAF50; font-weight: bold;');
    console.log('%cUser:', 'color: #2196F3; font-weight: bold;', user);
    console.log('%cToken:', 'color: #2196F3;', token ? '[present]' : null);
    console.log('%cProfile Picture URL:', 'color: #2196F3;', profilePictureUrl);
    console.log('%cLoading:', 'color: #2196F3;', loading);
    console.log('%cIs Admin:', 'color: #2196F3;', isAdmin);
    console.log('%cIs User:', 'color: #2196F3;', isUser);
    console.groupCollapsed('%cModules (Flat):', 'color: #FF9800; font-weight: bold;');
    console.table(modules);
    console.groupEnd();
    console.groupCollapsed('%cModules By Role:', 'color: #FF9800; font-weight: bold;');
    console.log('Lecturer:', modulesByRole.lecturer);
    console.log('Assistant Lecturer:', modulesByRole.assistant_lecturer);
    console.log('Tutor:', modulesByRole.tutor);
    console.log('Student:', modulesByRole.student);
    console.groupEnd();
    console.groupEnd();
  };

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
        token,
        isAdmin,
        isUser,
        getModuleRole,
        hasModuleRole,
        isLecturer,
        isAssistantLecturer,
        isTutor,
        isStudent,
        isStaff,
        hasLecturerRole,
        hasAssistantLecturerRole,
        hasTutorRole,
        hasStudentRole,
        hasStaffRole,
        print,
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
