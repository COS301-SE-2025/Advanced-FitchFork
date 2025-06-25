import { BrowserRouter as Router, Routes, Route, Navigate } from 'react-router-dom';
import { useAuth } from './context/AuthContext';

import Login from './pages/auth/Login';
import Signup from './pages/auth/Signup';
import RequestPasswordResetPage from './pages/auth/RequestPasswordResetPage';
import ResetPasswordPage from './pages/auth/ResetPasswordPage';
import PasswordResetSuccessPage from './pages/auth/PasswordResetSuccessPage';

import Forbidden from './pages/shared/status/Forbidden';
import Unauthorized from './pages/shared/status/Unauthorized';
import NotFound from './pages/shared/status/NotFound';

import Home from './pages/Home';
import UsersList from './pages/users/UsersList';
import UserView from './pages/users/UserView';
import UnderConstruction from './pages/shared/status/UnderConstruction';
import Modules from './pages/modules/index/Modules';
import CalendarPage from './pages/shared/CalendarPage';

import ModuleOverview from './pages/modules/show/ModuleOverview';
import ModulePersonnel from './pages/modules/show/ModulePersonnel';

import AppLayout from './layouts/AppLayout';
import ModuleLayout from './layouts/ModuleLayout';
import SettingsLayout from './layouts/SettingsLayout';
import Account from './pages/settings/Account';
import Security from './pages/settings/Security';
import Appearance from './pages/settings/Appearance';
import AssignmentLayout from './layouts/AssignmentLayout';
import SubmissionView from './pages/modules/assignments/submissions/show/SubmissionView';
import Submissions from './pages/modules/assignments/submissions/index/Submissions';
import SubmissionLayout from './layouts/SubmissionLayout';
import Assignments from './pages/modules/assignments/index/Assignments';

export default function App() {
  const { user, isAdmin, loading, isExpired } = useAuth();

  if (loading) return null;

  const requireAuth = (element: JSX.Element) =>
    user && !isExpired() ? element : <Navigate to="/login" replace />;

  const requireAdmin = (element: JSX.Element) =>
    user && !isExpired() ? (
      isAdmin ? (
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
        <Route path="/forgot-password" element={<RequestPasswordResetPage />} />
        <Route path="/reset-password" element={<ResetPasswordPage />} />
        <Route path="/password-reset-success" element={<PasswordResetSuccessPage />} />

        {/* Status + Fallback */}
        <Route path="/unauthorized" element={<Unauthorized />} />
        <Route path="/forbidden" element={<Forbidden />} />

        {/* AppLayout-wrapped Authenticated Routes */}
        <Route element={requireAuth(<AppLayout />)}>
          <Route path="/home" element={<Home />} />
          <Route path="/settings" element={<SettingsLayout />}>
            <Route index element={<Navigate to="account" replace />} />
            <Route path="account" element={<Account />} />
            <Route path="security" element={<Security />} />
            <Route path="appearance" element={<Appearance />} />
            {/* Add others here */}
          </Route>
          <Route path="/calendar" element={<CalendarPage />} />

          {/* Admin-only Routes */}
          <Route path="/users" element={requireAdmin(<UsersList />)} />
          <Route path="/users/:id" element={requireAdmin(<UserView />)} />
          <Route path="/users/:id/modules" element={requireAdmin(<Unauthorized />)} />

          {/* Modules Overview Pages */}
          <Route path="/modules" element={<Modules />} />

          {/* Module Layout Wrapper */}
          <Route path="/modules/:id" element={<ModuleLayout />}>
            <Route index element={<ModuleOverview />} />
            <Route path="assignments" element={<Assignments />} />
            <Route path="assignments/:assignment_id" element={<AssignmentLayout />}>
              <Route path="submissions" element={<Submissions />} />
              <Route path="tasks" element={<UnderConstruction />} />
              <Route path="config" element={<UnderConstruction />} />
              <Route path="stats" element={<UnderConstruction />} />
            </Route>
            <Route path="assignments/:assignment_id" element={<SubmissionLayout />}>
              <Route path="submissions/:submission_id" element={<SubmissionView />} />
            </Route>
            <Route path="grades" element={<UnderConstruction />} />
            <Route path="resources" element={<UnderConstruction />} />
            <Route path="personnel" element={<ModulePersonnel />} />
          </Route>

          {/* Fallback for assignment routes not implemented */}
          <Route path="/modules/:module_id/assignments" element={<Unauthorized />} />
          <Route path="/modules/:module_id/assignments/:assignment_id" element={<Unauthorized />} />

          <Route path="/reports" element={<UnderConstruction />} />
        </Route>

        <Route path="*" element={<NotFound />} />
      </Routes>
    </Router>
  );
}
