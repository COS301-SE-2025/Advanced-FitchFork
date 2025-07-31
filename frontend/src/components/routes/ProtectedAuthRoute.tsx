import { Navigate, Outlet } from 'react-router-dom';
import { useAuth } from '@/context/AuthContext';

export default function ProtectedAuthRoute() {
  const { user, isExpired } = useAuth();

  if (!user || isExpired()) return <Navigate to="/login" replace />;

  return <Outlet />;
}
