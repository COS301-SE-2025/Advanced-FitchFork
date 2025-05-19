import { BrowserRouter as Router, Routes, Route, Navigate } from 'react-router-dom';
import { useAuth } from './context/AuthContext';
import Login from '@pages/auth/Login';
import Signup from '@pages/auth/Signup';
import Dashboard from '@pages/Dashboard';
import Unauthorized from '@pages/status/Unauthorized';
import { ProtectedRoute } from './routes/ProtectedRoute';
import { UserRole } from '@models/auth';
import NotFound from '@pages/status/NotFound';
import ServerError from '@pages/status/ServerError';

function AppRoutes() {
  const { user } = useAuth();

  return (
    <Routes>
      {/* Default redirect */}
      <Route path="/" element={<Navigate to={user ? '/dashboard' : '/login'} />} />

      {/* Public routes */}
      <Route path="/login" element={<Login />} />
      <Route path="/signup" element={<Signup />} />
      <Route path="/unauthorized" element={<Unauthorized />} />

      {/* Protected route */}
      <Route
        path="/dashboard"
        element={
          <ProtectedRoute requiredRoles={[UserRole.Admin]}>
            <Dashboard />
          </ProtectedRoute>
        }
      />
      <Route path="internal-error" element={<ServerError />} />
      <Route path="*" element={<NotFound />} />
    </Routes>
  );
}

export default function App() {
  return (
    <Router>
      <AppRoutes />
    </Router>
  );
}
