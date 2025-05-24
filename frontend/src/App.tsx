import { BrowserRouter as Router, Routes, Route, Navigate } from 'react-router-dom';
import { useAuth } from './context/AuthContext';
import AppLayout from './layouts/AppLayout';
import Login from './pages/auth/Login';
import Signup from './pages/auth/Signup';
import Home from './pages/Home';
import UserModuleGridView from './pages/modules/UserModuleGridView';
import NotFound from './pages/status/NotFound';
import ServerError from './pages/status/ServerError';
import Unauthorized from './pages/status/Unauthorized';
import UsersList from './pages/users/UsersList';
import UserView from './pages/users/UserView';
import { ProtectedRoute } from './routes/ProtectedRoute';
import ProfilePage from './pages/Profile';
import ModuleList from './pages/admin/modules/ModuleList';
import ModuleView from './pages/admin/modules/ModuleView';

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
      <Route path="/home" element={<Home />} />
      <Route path="/profile" element={<ProfilePage />} />
      <Route
        path="/settings"
        element={
          <AppLayout title="Settings">
            <div>Settings Page</div>
          </AppLayout>
        }
      />
      <Route
        path="/users"
        element={
          <ProtectedRoute requiredAdmin={true}>
            <UsersList />
          </ProtectedRoute>
        }
      />
      <Route
        path="/users/:id"
        element={
          <ProtectedRoute requiredAdmin={true}>
            <UserView />
          </ProtectedRoute>
        }
      />
      <Route path="/modules" element={<ModuleList />} /> {/* Admin */}
      <Route path="/modules/:id" element={<ModuleView />} />
      <Route path="/modules/enrolled" element={<UserModuleGridView filter="student" />} />
      <Route path="/modules/tutoring" element={<UserModuleGridView filter="tutor" />} />
      <Route path="/modules/lecturing" element={<UserModuleGridView filter="lecturer" />} />
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
