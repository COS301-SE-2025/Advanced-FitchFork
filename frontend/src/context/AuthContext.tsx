import { AuthService } from '@/services/auth';
import type { AuthUser, LoginRequest, RegisterRequest, UserModule } from '@/types/auth';
import type { User } from '@/types/users';
import type { ApiResponse } from '@/utils/api';
import React, { createContext, useContext, useState, useEffect } from 'react';

type ModuleRole = 'Lecturer' | 'Tutor' | 'Student';

interface AuthContextType {
  user: User | null;
  modules: UserModule[];
  loading: boolean;
  login: (credentials: LoginRequest) => Promise<ApiResponse<AuthUser | null>>;
  register: (details: RegisterRequest) => Promise<ApiResponse<AuthUser | null>>;
  logout: () => void;
  isAdmin: () => boolean;
  getModuleRole: (moduleId: number) => ModuleRole | null;
  hasModuleRole: (moduleId: number, role: ModuleRole) => boolean;
}

const AuthContext = createContext<AuthContextType | undefined>(undefined);

export const AuthProvider: React.FC<{ children: React.ReactNode }> = ({ children }) => {
  const [user, setUser] = useState<User | null>(null);
  const [modules, setModules] = useState<UserModule[]>([]);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    const stored = localStorage.getItem('auth');
    if (stored) {
      try {
        const parsed = JSON.parse(stored);

        // Check expiry
        if (parsed.expires_at && new Date(parsed.expires_at) < new Date()) {
          logout(); // token expired
          return;
        }

        if (parsed.user) setUser(parsed.user);
        if (parsed.modules) setModules(parsed.modules);
      } catch {
        // Corrupt data or parsing failed
        logout();
      }
    }
    setLoading(false);
  }, []);

  const login = async (credentials: LoginRequest): Promise<ApiResponse<AuthUser | null>> => {
    try {
      const res = await AuthService.login(credentials);
      if (res.success && res.data) {
        const { token, expires_at, ...user } = res.data;

        setUser(user);
        setModules([]); // You can update this later with `me`

        localStorage.setItem(
          'auth',
          JSON.stringify({
            user,
            modules: [],
            token,
            expires_at,
          }),
        );
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

  const register = async (details: RegisterRequest): Promise<ApiResponse<AuthUser | null>> => {
    try {
      const res = await AuthService.register(details);
      return res;
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
  };

  const isAdmin = () => !!user?.admin;

  const getModuleRole = (moduleId: number): ModuleRole | null => {
    const mod = modules.find((m) => m.id === moduleId);
    return mod?.role || null;
  };

  const hasModuleRole = (moduleId: number, role: ModuleRole): boolean => {
    return getModuleRole(moduleId) === role;
  };

  return (
    <AuthContext.Provider
      value={{
        user,
        modules,
        loading,
        login,
        register,
        logout,
        isAdmin,
        getModuleRole,
        hasModuleRole,
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
