import React, { createContext, useContext, useState, useEffect } from 'react';
import type { User, UserRole } from '../types/auth';

/**
 * Describes the shape of the authentication context.
 * - `user`: The current authenticated user or `null`.
 * - `login`: Function to set the user and persist it in localStorage.
 * - `logout`: Clears the user state and removes it from localStorage.
 * - `hasRole`: Utility function to check if the user has a given role.
 */
interface AuthContextType {
  user: User | null;
  login: (user: User) => void;
  logout: () => void;
  hasRole: (role: UserRole) => boolean;
}

/**
 * Create the context with an initial value of `undefined`.
 * It will be checked inside the `useAuth` hook to ensure it is used properly.
 */
const AuthContext = createContext<AuthContextType | undefined>(undefined);

/**
 * Provides authentication state and logic to the app.
 *
 * Wrap your application in `AuthProvider` (typically in `main.tsx`) to:
 * - Track the current user
 * - Persist user info across page reloads
 * - Provide login/logout and role checking
 */
export const AuthProvider: React.FC<{ children: React.ReactNode }> = ({ children }) => {
  const [user, setUser] = useState<User | null>(null);

  /**
   * On initial load, restore auth state from localStorage (if present).
   * This enables persistent login without a server session.
   */
  useEffect(() => {
    const stored = localStorage.getItem('auth');
    if (stored) {
      setUser(JSON.parse(stored));
    }
  }, []);

  /**
   * Log in a user by storing their data in state and localStorage.
   * @param userData - The user object (must include token and roles)
   */
  const login = (userData: User) => {
    localStorage.setItem('auth', JSON.stringify(userData));
    setUser(userData);
  };

  /**
   * Logs out the user by clearing state and removing localStorage entry.
   */
  const logout = () => {
    localStorage.removeItem('auth');
    setUser(null);
  };

  /**
   * Check if the current user has a specific role.
   * @param role - A string like "admin", "tutor", "student"
   * @returns `true` if the user has the role, else `false`
   */
  const hasRole = (role: UserRole): boolean => {
    return !!user?.roles.includes(role);
  };

  /**
   * Provide the auth state and functions to all children components.
   */
  return (
    <AuthContext.Provider value={{ user, login, logout, hasRole }}>{children}</AuthContext.Provider>
  );
};

/**
 * Custom hook to access auth context from any component.
 * Throws an error if used outside an `AuthProvider`.
 *
 * @returns The authentication context (`user`, `login`, `logout`, `hasRole`)
 */
export const useAuth = (): AuthContextType => {
  const ctx = useContext(AuthContext);
  if (!ctx) throw new Error('useAuth must be used within AuthProvider');
  return ctx;
};
