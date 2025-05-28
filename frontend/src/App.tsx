import { BrowserRouter as Router, Routes, Route, Navigate } from 'react-router-dom';
import { useAuth } from './context/AuthContext';
import Login from './pages/auth/Login';
import Signup from './pages/auth/Signup';
import Forbidden from './pages/shared/status/Forbidden';
import Unauthorized from './pages/shared/status/Unauthorized';
import Home from './pages/Home';
import UsersList from './pages/users/UsersList';
import ModuleList from './pages/modules/admin/ModuleList';
import ModuleView from './pages/modules/admin/view/ModuleView';
import NotFound from './pages/shared/status/NotFound';
import UnderConstruction from './pages/shared/status/UnderConstruction';
import ProfilePage from './pages/shared/Profile';
import UserView from './pages/users/UserView';

export default function App() {
  const { user, isAdmin, loading, isExpired } = useAuth();

  if (loading) return null;

  const requireAuth = (element: JSX.Element) =>
    user && !isExpired() ? element : <Navigate to="/login" replace />;

  const requireAdmin = (element: JSX.Element) =>
    user && !isExpired() ? (
      isAdmin() ? (
        element
      ) : (
        <Navigate to="/unauthorized" replace />
      )
    ) : (
      <Navigate to="/forbidden" replace />
    );

  return (
    <Router>
      <Routes>
        {/* Public Auth Routes */}
        <Route path="/" element={<Navigate to={user ? '/home' : '/login'} />} />
        <Route path="/login" element={<Login />} />
        <Route path="/signup" element={<Signup />} />

        {/* Admin-only User Routes */}
        <Route path="/users" element={requireAdmin(<UsersList />)} />
        <Route path="/users/:id" element={requireAdmin(<UserView />)} />
        <Route path="/users/:id/modules" element={requireAdmin(<Unauthorized />)} />

        <Route path="/home" element={requireAuth(<Home />)} />
        <Route path="/settings" element={requireAuth(<UnderConstruction />)} />
        <Route path="/profile" element={requireAuth(<ProfilePage />)} />

        {/* Modules */}
        <Route path="/modules" element={requireAuth(<ModuleList />)} />
        <Route path="/modules/my" element={requireAuth(<Unauthorized />)} />
        <Route path="/modules/:id" element={requireAuth(<ModuleView />)} />
        <Route path="/modules/:id/edit" element={requireAuth(<Unauthorized />)} />

        {/* Assignments */}
        <Route path="/modules/:module_id/assignments" element={requireAuth(<Unauthorized />)} />
        <Route
          path="/modules/:module_id/assignments/:assignment_id"
          element={requireAuth(<Unauthorized />)}
        />

        <Route path="/reports" element={<UnderConstruction />} />

        <Route path="/unauthorized" element={<Unauthorized />} />
        <Route path="/forbidden" element={<Forbidden />} />
        {/* Fallback */}
        <Route path="*" element={<NotFound />} />
      </Routes>
    </Router>
  );
}
