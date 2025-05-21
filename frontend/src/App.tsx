import { BrowserRouter as Router, Routes, Route, Navigate } from 'react-router-dom';
import { useAuth } from './context/AuthContext';
import Login from '@pages/auth/Login';
import Signup from '@pages/auth/Signup';
import Dashboard from '@pages/Dashboard';
import Unauthorized from '@pages/status/Unauthorized';
import NotFound from '@pages/status/NotFound';
import ServerError from '@pages/status/ServerError';
import DashboardLayout from '@layouts/DashboardLayout';
import UserView from '@pages/users/UserView';
import UserEdit from '@pages/users/UserEdit';
import UsersList from '@pages/users/UsersList';

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

      <Route path="/dashboard" element={<Dashboard />} />
      <Route
        path="/dashboard/settings"
        element={
          <DashboardLayout title="Settings">
            <div>Settings Page</div>
          </DashboardLayout>
        }
      />
      <Route
        path="/dashboard/submission-history"
        element={
          <DashboardLayout title="Submissions">
            <div>Submissions</div>
          </DashboardLayout>
        }
      />
      <Route path="/users" element={<UsersList />} />
      <Route path="/users/:id" element={<UserView />} />
      <Route path="/users/:id/edit" element={<UserEdit />} />

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
