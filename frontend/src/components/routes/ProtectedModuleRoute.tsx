import { Navigate, Outlet, useParams } from 'react-router-dom';
import { useAuth } from '../../context/AuthContext';
import type { ModuleRole } from '../../types/modules';
import type { PropsWithChildren } from 'react';

interface Props {
  allowedRoles: ModuleRole[];
}

export default function ProtectedModuleRoute({ allowedRoles, children }: PropsWithChildren<Props>) {
  const { id } = useParams();
  const { isExpired, user, isAdmin, getModuleRole } = useAuth();

  if (!user || isExpired()) return <Navigate to="/login" replace />;

  if (isAdmin) return <>{children ?? <Outlet />}</>;

  const moduleId = parseInt(id ?? '', 10);
  const role = getModuleRole(moduleId);

  if (!role || !allowedRoles.includes(role)) {
    return <Navigate to="/unauthorized" replace />;
  }

  return <>{children ?? <Outlet />}</>;
}
