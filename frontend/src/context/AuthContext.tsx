import { AuthService } from '@/services/auth';
import type { LoginRequest, RegisterRequest, LoginResponse, RegisterResponse } from '@/types/auth';
import type { User } from '@/types/users';
import type { ApiResponse } from '@/utils/api';
import React, { createContext, useContext, useState, useEffect } from 'react';

interface AuthContextType {
  user: User | null;
  login: (credentials: LoginRequest) => Promise<ApiResponse<LoginResponse>>;
  register: (details: RegisterRequest) => Promise<ApiResponse<RegisterResponse>>;
  logout: () => void;
  isAdmin: () => boolean;
  // getModuleRole: (moduleId: number) => ModuleRole | null;
  // hasModuleRole: (moduleId: number, role: ModuleRole) => boolean;
}

const AuthContext = createContext<AuthContextType | undefined>(undefined);

export const AuthProvider: React.FC<{ children: React.ReactNode }> = ({ children }) => {
  const [user, setUser] = useState<User | null>(null);

  useEffect(() => {
    const stored = localStorage.getItem('auth');
    if (stored) {
      setUser(JSON.parse(stored));
    }
  }, []);

  const login = async (credentials: LoginRequest): Promise<ApiResponse<LoginResponse>> => {
    try {
      const res = await AuthService.login(credentials);
      if (res.success && res.data) {
        localStorage.setItem('auth', JSON.stringify(res.data));
        setUser(res.data);
      }
      return res;
    } catch (err: any) {
      return {
        success: false,
        message: err.message || 'Unexpected error during login',
      } as ApiResponse<LoginResponse>;
    }
  };

  const register = async (details: RegisterRequest): Promise<ApiResponse<RegisterResponse>> => {
    try {
      const res = await AuthService.register(details);
      if (res.success && res.data) {
        localStorage.setItem('auth', JSON.stringify(res.data));
        setUser(res.data);
      }
      return res;
    } catch (err: any) {
      return {
        success: false,
        message: err.message || 'Unexpected error during registration',
      } as ApiResponse<RegisterResponse>;
    }
  };

  const logout = () => {
    localStorage.removeItem('auth');
    setUser(null);
  };

  const isAdmin = () => !!user?.admin;

  // const getModuleRole = (moduleId: number): ModuleRole | null => null;
  // const hasModuleRole = (moduleId: number, role: ModuleRole) => false;

  return (
    <AuthContext.Provider value={{ user, login, register, logout, isAdmin }}>
      {children}
    </AuthContext.Provider>
  );
};

export const useAuth = (): AuthContextType => {
  const ctx = useContext(AuthContext);
  if (!ctx) throw new Error('useAuth must be used within AuthProvider');
  return ctx;
};
