import React from 'react';
import { Navigate } from 'react-router-dom';
import { useAuth } from '../context/AuthContext';

interface ProtectedRouteProps {
  children: React.ReactNode;
  requiredAdmin?: boolean;
}

export const ProtectedRoute: React.FC<ProtectedRouteProps> = ({
  children,
  requiredAdmin = false,
}) => {
  const { user, isAdmin } = useAuth();

  // Not logged in
  if (!user) return <Navigate to="/login" replace />;

  // Admin access required
  if (requiredAdmin && isAdmin) return <Navigate to="/unauthorized" replace />;

  // Membership check
  // if (requiredMemberships.length > 0) {
  //   const hasAccess = requiredMemberships.some(({ moduleId, role }) =>
  //     hasModuleRole(moduleId, role),
  //   );
  //   if (!hasAccess) return <Navigate to="/unauthorized" replace />;
  // }

  return <>{children}</>;
};
