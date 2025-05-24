import React from 'react';
import { Navigate } from 'react-router-dom';
import { useAuth } from '../context/AuthContext';
import type { ModuleMembership } from '@/types/users';

interface ProtectedRouteProps {
  children: React.ReactNode;
  requiredAdmin?: boolean;
  requiredMemberships?: ModuleMembership[];
}

export const ProtectedRoute: React.FC<ProtectedRouteProps> = ({
  children,
  requiredAdmin = false,
  requiredMemberships = [],
}) => {
  const { user, hasModuleRole, isAdmin } = useAuth();

  // Not logged in
  if (!user) return <Navigate to="/login" replace />;

  // Admin access required
  if (requiredAdmin && !isAdmin()) return <Navigate to="/unauthorized" replace />;

  // Membership check
  if (requiredMemberships.length > 0) {
    const hasAccess = requiredMemberships.some(({ moduleId, role }) =>
      hasModuleRole(moduleId, role),
    );
    if (!hasAccess) return <Navigate to="/unauthorized" replace />;
  }

  return <>{children}</>;
};
