import { AuthService } from '@/services/auth';
import type { AuthUser, LoginRequest, RegisterRequest, UserModule } from '@/types/auth';
import type { User } from '@/types/users';
import type { ApiResponse } from '@/utils/api';
import React, { createContext, useContext, useState, useEffect } from 'react';

type ModuleRole = 'Lecturer' | 'Tutor' | 'Student';

/**
 * Defines the structure of the authentication context.
 */
interface AuthContextType {
  /** The currently authenticated user or null. */
  user: User | null;

  /** The list of modules assigned to the current user. */
  modules: UserModule[];

  /** Whether the context is still loading user data. */
  loading: boolean;

  /** Logs the user in with given credentials. */
  login: (credentials: LoginRequest) => Promise<ApiResponse<AuthUser | null>>;

  /** Registers a new user with the given details. */
  register: (details: RegisterRequest) => Promise<ApiResponse<AuthUser | null>>;

  /** Logs the user out and clears stored auth info. */
  logout: () => void;

  /** Returns whether the current user is an admin. */
  isAdmin: () => boolean;

  /** Returns the role of the user in a specific module. */
  getModuleRole: (moduleId: number) => ModuleRole | null;

  /** Returns true if the user has the specified role in a module. */
  hasModuleRole: (moduleId: number, role: ModuleRole) => boolean;

  /** Returns true if the stored token is expired. */
  isExpired: () => boolean;
}

const AuthContext = createContext<AuthContextType | undefined>(undefined);

/**
 * Provides authentication state and actions to the app.
 */
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

          if (isExpired()) {
            logout();
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
        } catch (err) {
          logout(); // Token expired or fetch error
        }
      }
      setLoading(false);
    };

    loadUser();
  }, []);

  /**
   * Logs in a user and saves their session to localStorage.
   */
  const login = async (credentials: LoginRequest): Promise<ApiResponse<AuthUser | null>> => {
    try {
      const res = await AuthService.login(credentials);
      if (res.success && res.data) {
        const { token, expires_at, ...user } = res.data;

        localStorage.setItem(
          'auth',
          JSON.stringify({
            user,
            modules: [],
            token,
            expires_at,
          }),
        );

        const meRes = await AuthService.me();
        if (meRes.success && meRes.data) {
          const { modules: userModules, ...userData } = meRes.data;
          setUser(userData);
          setModules(userModules);
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

  /**
   * Registers a new user and returns the API response.
   */
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

  /**
   * Logs the user out and redirects to the login page.
   */
  const logout = () => {
    localStorage.removeItem('auth');
    setUser(null);
    setModules([]);
    window.location.href = '/login';
  };

  /**
   * Checks if the stored token is expired based on its expiration timestamp.
   */
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

  /**
   * Returns true if the user has admin privileges.
   */
  const isAdmin = () => !!user?.admin;

  /**
   * Gets the role of the user for the given module ID.
   */
  const getModuleRole = (moduleId: number): ModuleRole | null => {
    const mod = modules.find((m) => m.id === moduleId);
    return mod?.role || null;
  };

  /**
   * Checks if the user has a specific role in the given module.
   */
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
        isExpired,
      }}
    >
      {children}
    </AuthContext.Provider>
  );
};

/**
 * Hook to access the authentication context.
 */
export const useAuth = (): AuthContextType => {
  const ctx = useContext(AuthContext);
  if (!ctx) throw new Error('useAuth must be used within AuthProvider');
  return ctx;
};

export default AuthProvider;
