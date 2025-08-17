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

import UsersList from './pages/users/UsersList';
import UserView from './pages/users/UserView';
import UnderConstruction from './pages/shared/status/UnderConstruction';
import CalendarPage from './pages/shared/CalendarPage';

import ModuleOverview from './pages/modules/ModuleOverview';
import ModulePersonnel from './pages/modules/ModulePersonnel';

import AppLayout from './layouts/AppLayout';
import ModuleLayout from './layouts/ModuleLayout';
import SettingsLayout from './layouts/SettingsLayout';
import Account from './pages/settings/Account';
import Security from './pages/settings/Security';
import Appearance from './pages/settings/Appearance';
import AssignmentLayout from './layouts/AssignmentLayout';
import SubmissionView from './pages/modules/assignments/submissions/SubmissionView';
import AssignmentFiles from './pages/modules/assignments/AssignmentFiles';
import MemoOutput from './pages/modules/assignments/MemoOutput';
import MarkAllocator from './pages/modules/assignments/MarkAllocator';
import HelpPageLayout from './layouts/HelpPageLayout';
import HelpAccount from './pages/help/HelpAccount';
import HelpAssignments from './pages/help/HelpAssignments';
import HelpContact from './pages/help/HelpContact';
import HelpSubmissions from './pages/help/HelpSubmissions';
import HelpTroubleshooting from './pages/help/HelpTroubleshooting';
import Landing from './pages/Landing';
import Dashboard from './pages/Dashboard';
import Tasks from './pages/modules/assignments/Tasks';
import AuthLayout from './layouts/AuthLayout';
import ModulesList from './pages/modules/ModulesList';
import ModuleGrades from './pages/modules/ModuleGrades';
import AssignmentsList from './pages/modules/assignments/AssignmentsList';

import ProtectedAuthRoute from './components/routes/ProtectedAuthRoute';
import ProtectedAdminRoute from './components/routes/ProtectedAdminRoute';
import ProtectedModuleRoute from './components/routes/ProtectedModuleRoute';
import Tickets from './pages/modules/assignments/tickets/Tickets';
import TicketView from './pages/modules/assignments/tickets/TicketView';
import WithModuleContext from './components/providers/WithModuleContext';
import WithAssignmentContext from './components/providers/WithAssignmentContext';
import AssignmentConfigLayout from './layouts/ConfigLayout';
import ExecutionPage from './pages/modules/assignments/config/ExecutionPage';
import MarkingPage from './pages/modules/assignments/config/MarkingPage';
import { useUI } from './context/UIContext';
import AssignmentMobileMenu from './pages/modules/assignments/AssignmentMobileMenu';
import ModuleMobileMenu from './pages/modules/ModuleMobileMenu';
import ConfigMobileMenu from './pages/modules/assignments/config/ConfigMobileMenu';
import SubmissionsList from './pages/modules/assignments/submissions/SubmissionsList';
import SettingsMobileMenu from './pages/settings/SettingsMobileMenu';
import Announcements from './pages/modules/announcements/Announcements';
import AnnouncementView from './pages/modules/announcements/AnnouncementView';
import PlagiarismCases from './pages/modules/assignments/PlagiarismCases';

export default function App() {
  const { isMobile } = useUI();
  const { user, loading, isExpired } = useAuth();

  if (loading) return null;

  return (
    <Router>
      <Routes>
        {/* Public Auth Routes */}
        <Route
          path="/"
          element={user && !isExpired() ? <Navigate to="/dashboard" replace /> : <Landing />}
        />
        <Route element={<AuthLayout />}>
          <Route path="/login" element={<Login />} />
          <Route path="/signup" element={<Signup />} />
          <Route path="/forgot-password" element={<RequestPasswordResetPage />} />
          <Route path="/reset-password" element={<ResetPasswordPage />} />
          <Route path="/password-reset-success" element={<PasswordResetSuccessPage />} />
        </Route>

        {/* Status + Fallback */}
        <Route path="/unauthorized" element={<Unauthorized />} />
        <Route path="/forbidden" element={<Forbidden />} />
        <Route path="*" element={<NotFound />} />

        {/* Protected Auth Routes */}
        <Route element={<ProtectedAuthRoute />}>
          <Route element={<AppLayout />}>
            <Route path="/dashboard" element={<Dashboard />} />

            <Route path="/settings" element={<SettingsLayout />}>
              <Route
                index
                element={isMobile ? <SettingsMobileMenu /> : <Navigate to="account" replace />}
              />
              <Route path="account" element={<Account />} />
              <Route path="security" element={<Security />} />
              <Route path="appearance" element={<Appearance />} />
            </Route>

            <Route path="/calendar" element={<CalendarPage />} />

            {/* Admin-only routes */}
            <Route element={<ProtectedAdminRoute />}>
              <Route path="/users" element={<UsersList />} />
              <Route path="/users/:id" element={<UserView />} />
              <Route path="/users/:id/modules" element={<Unauthorized />} />
            </Route>

            {/* Modules */}
            <Route path="/modules" element={<ModulesList />} />
            <Route path="/modules/:id" element={<WithModuleContext />}>
              <Route path="/modules/:id" element={<ModuleLayout />}>
                <Route
                  index
                  element={isMobile ? <ModuleMobileMenu /> : <Navigate to="overview" replace />}
                />
                <Route path="overview" element={<ModuleOverview />} />

                <Route path="announcements">
                  <Route index element={<Announcements />} />
                  <Route path=":announcement_id" element={<AnnouncementView />} />
                </Route>
                <Route path="assignments" element={<AssignmentsList />} />
                <Route path="assignments/:assignment_id" element={<WithAssignmentContext />}>
                  <Route element={<AssignmentLayout />}>
                    <Route
                      index
                      element={
                        isMobile ? <AssignmentMobileMenu /> : <Navigate to="submissions" replace />
                      }
                    />
                    <Route path="files" element={<AssignmentFiles />} />
                    <Route path="submissions" element={<SubmissionsList />} />
                    <Route path="submissions/:submission_id" element={<SubmissionView />} />
                    <Route path="tasks" element={<Tasks />}>
                      <Route index element={<></>} />
                      <Route path=":task_id" element={<></>} />
                    </Route>
                    <Route path="tickets" element={<Tickets />} />
                    <Route path="memo-output" element={<MemoOutput />} />
                    <Route path="mark-allocator" element={<MarkAllocator />} />
                    <Route path="stats" element={<UnderConstruction />} />

                    <Route path="plagiarism">
                      <Route index element={<PlagiarismCases />} />
                      <Route path=":plagiarism_id" element={<></>} />
                    </Route>
                    <Route path="config" element={<AssignmentConfigLayout />}>
                      <Route
                        index
                        element={
                          isMobile ? <ConfigMobileMenu /> : <Navigate to="execution" replace />
                        }
                      />
                      <Route path="execution" element={<ExecutionPage />} />
                      <Route path="marking" element={<MarkingPage />} />
                    </Route>
                  </Route>

                  <Route path="tickets/:ticket_id" element={<TicketView />} />
                </Route>

                <Route path="bookings" element={<UnderConstruction />} />
                <Route path="grades" element={<ModuleGrades />} />
                <Route path="resources" element={<UnderConstruction />} />
                <Route
                  path="personnel"
                  element={
                    <ProtectedModuleRoute allowedRoles={['lecturer', 'assistant_lecturer']}>
                      <ModulePersonnel />
                    </ProtectedModuleRoute>
                  }
                />
              </Route>
            </Route>

            {/* Help Routes */}
            <Route path="/help" element={<HelpPageLayout />}>
              <Route index element={<Navigate to="account" replace />} />
              <Route path="account" element={<HelpAccount />} />
              <Route path="assignments" element={<HelpAssignments />} />
              <Route path="submissions" element={<HelpSubmissions />} />
              <Route path="troubleshooting" element={<HelpTroubleshooting />} />
              <Route path="contact" element={<HelpContact />} />
            </Route>

            {/* Explicit Unauthorized Fallbacks */}
            <Route path="/modules/:module_id/assignments" element={<Unauthorized />} />
            <Route
              path="/modules/:module_id/assignments/:assignment_id"
              element={<Unauthorized />}
            />
            <Route path="/reports" element={<UnderConstruction />} />
          </Route>
        </Route>
      </Routes>
    </Router>
  );
}
