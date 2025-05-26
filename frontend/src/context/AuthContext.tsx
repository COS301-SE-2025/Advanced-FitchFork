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
    const loadUser = async () => {
      const stored = localStorage.getItem('auth');
      if (stored) {
        try {
          const parsed = JSON.parse(stored);
          const { token, expires_at } = parsed;

          if (!token || (expires_at && new Date(expires_at) < new Date())) {
            logout();
            return;
          }

          const res = await AuthService.me();
          if (res.success && res.data) {
            const { modules: userModules, ...userData } = res.data;
            setUser(userData);
            setModules(userModules);
            localStorage.setItem(
              'auth',
              JSON.stringify({
                ...parsed,
                user: userData,
                modules: userModules,
              }),
            );
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

  const login = async (credentials: LoginRequest): Promise<ApiResponse<AuthUser | null>> => {
    try {
      const res = await AuthService.login(credentials);
      if (res.success && res.data) {
        const { token, expires_at, ...user } = res.data;

        // Save token first
        localStorage.setItem(
          'auth',
          JSON.stringify({
            user,
            modules: [],
            token,
            expires_at,
          }),
        );

        // Then call /me using the saved token
        const meRes = await AuthService.me();
        if (meRes.success && meRes.data) {
          const { modules: userModules, ...userData } = meRes.data;
          setUser(userData);
          setModules(userModules);

          // Save updated info again
          localStorage.setItem(
            'auth',
            JSON.stringify({
              user: userData,
              modules: userModules,
              token,
              expires_at,
            }),
          );
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
