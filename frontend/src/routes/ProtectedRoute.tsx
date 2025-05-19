import React from 'react';
import { Navigate } from 'react-router-dom';
import { useAuth } from '../context/AuthContext';
import type { UserRole } from '../types/auth';

interface ProtectedRouteProps {
  children: React.ReactNode;
  requiredRoles?: UserRole[]; // updated to be strongly typed
}

export const ProtectedRoute: React.FC<ProtectedRouteProps> = ({ children, requiredRoles }) => {
  const { user, hasRole } = useAuth();

  if (!user) return <Navigate to="/login" />;

  if (requiredRoles && !requiredRoles.some(hasRole)) {
    return <Navigate to="/unauthorized" />;
  }

  return children;
};
