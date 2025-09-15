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
import Landing from './pages/Landing';
import Dashboard from './pages/Dashboard';
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
import AssignmentPage from './pages/modules/assignments/config/AssignmentPage';
import OutputPage from './pages/modules/assignments/config/OutputPage';
import GatlamPage from './pages/modules/assignments/config/GatlamPage';
import CodeCoveragePage from './pages/modules/assignments/config/CodeCoveragePage';
import InterpreterPage from './pages/modules/assignments/config/InterpreterPage';
import AssignmentFilePage from './pages/modules/assignments/config/AssignmentFilePage';
import SubmissionIde from './pages/modules/assignments/submissions/SubmissionIde';
import AssignmentGrades from './pages/modules/assignments/AssignmentGrades';
import AttendanceSessionsList from './pages/modules/attendance/AttendanceSessionsList';
import AttendanceSessionView from './pages/modules/attendance/AttendanceSessionView';
import AttendanceMarkPage from './pages/modules/attendance/AttendanceMarkPage';
import AttendanceSessionProjector from './pages/modules/attendance/AttendanceSessionProjector';
import SecurityPage from './pages/modules/assignments/config/SecurityPage';
import AssignmentVerifyPage from './pages/modules/assignments/AssignmentVerifyPage';
import AccessDeniedPage from './pages/modules/assignments/AccessDeniedPage';
import TaskView from './pages/modules/assignments/tasks/TaskView';
import TasksPage from './pages/modules/assignments/tasks';
import MakefileHelp from './pages/help/assignments/files/Makefile';
import MainFile from './pages/help/assignments/files/MainFile';
import MemoFiles from './pages/help/assignments/files/MemoFiles';
import Specification from './pages/help/assignments/files/Specification';
import ExecutionHelp from './pages/help/assignments/config/ExecutionHelp';
import OutputHelp from './pages/help/assignments/config/OutputHelp';
import MarkingHelp from './pages/help/assignments/config/MarkingHelp';
import ProjectHelp from './pages/help/assignments/config/ProjectHelp';
import SecurityHelp from './pages/help/assignments/config/SecurityHelp';
import GATLAMHelp from './pages/help/assignments/config/GATLAMHelp';
import ConceptGATLAM from './pages/help/assignments/gatlam/ConceptGATLAM';
import ConceptCodeCoverage from './pages/help/assignments/coverage/ConceptCodeCoverage';
import ConfigOverviewHelp from './pages/help/assignments/config/ConfigOverviewHelp';
import MemoOutputHelp from './pages/help/assignments/MemoOutputHelp';
import MarkAllocatorHelp from './pages/help/assignments/MarkAllocatorHelp';
import HowToSubmitHelp from './pages/help/assignments/submissions/HowToSubmitHelp';
import AssignmentSetupHelp from './pages/help/assignments/AssignmentSetupHelp';
import TasksHelp from './pages/help/assignments/TasksHelp';

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

        {/* Auth-only pages not wrapped in AppLayout */}
        <Route element={<ProtectedAuthRoute />}>
          <Route path="/attendance/mark" element={<AttendanceMarkPage />} />
        </Route>

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
                {/* Verify page (does NOT use WithAssignmentContext) */}
                <Route
                  path="/modules/:id/assignments/:assignment_id/verify"
                  element={<AssignmentVerifyPage />}
                />

                <Route
                  path="assignments/:assignment_id/access-denied"
                  element={<AccessDeniedPage />}
                />

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
                    <Route path="tasks" element={<TasksPage />}>
                      <Route index element={<TaskView />} />
                      <Route path=":task_id" element={<TaskView />} />
                    </Route>
                    <Route path="tickets" element={<Tickets />} />
                    <Route path="memo-output" element={<MemoOutput />} />
                    <Route path="mark-allocator" element={<MarkAllocator />} />
                    <Route path="stats" element={<UnderConstruction />} />
                    <Route path="grades" element={<AssignmentGrades />} />

                    <Route path="plagiarism">
                      <Route index element={<PlagiarismCases />} />
                      <Route path=":plagiarism_id" element={<></>} />
                    </Route>
                    <Route path="config" element={<AssignmentConfigLayout />}>
                      <Route
                        index
                        element={
                          isMobile ? <ConfigMobileMenu /> : <Navigate to="assignment" replace />
                        }
                      />
                      <Route path="assignment" element={<AssignmentPage />} />
                      <Route path="execution" element={<ExecutionPage />} />
                      <Route path="marking" element={<MarkingPage />} />
                      <Route path="output" element={<OutputPage />} />
                      <Route path="security" element={<SecurityPage />} />
                      <Route path="code-coverage" element={<CodeCoveragePage />} />
                      <Route path="gatlam" element={<GatlamPage />} />
                      <Route path="interpreter" element={<InterpreterPage />} />
                      <Route path="files/:fileType" element={<AssignmentFilePage />} />
                    </Route>
                  </Route>

                  <Route path="tickets/:ticket_id" element={<TicketView />} />
                </Route>

                <Route
                  path="assignments/:assignment_id/submissions/:submission_id/code"
                  element={<SubmissionIde />}
                />

                <Route
                  path="attendance"
                  element={
                    <ProtectedModuleRoute allowedRoles={['lecturer', 'assistant_lecturer']}>
                      <AttendanceSessionsList />
                    </ProtectedModuleRoute>
                  }
                />

                <Route
                  path="attendance/sessions/:session_id"
                  element={
                    <ProtectedModuleRoute
                      allowedRoles={['lecturer', 'assistant_lecturer', 'tutor', 'student']}
                    >
                      <AttendanceSessionView />
                    </ProtectedModuleRoute>
                  }
                />

                <Route
                  path="/modules/:id/attendance/sessions/:session_id/projector"
                  element={
                    <ProtectedModuleRoute allowedRoles={['lecturer', 'assistant_lecturer']}>
                      <AttendanceSessionProjector />
                    </ProtectedModuleRoute>
                  }
                />

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
              {/* Default → Getting Started / Overview */}
              <Route index element={<Navigate to="getting-started/overview" replace />} />

              {/* Ambiguous roots → first leaf */}
              <Route
                path="getting-started"
                element={<Navigate to="getting-started/overview" replace />}
              />
              <Route path="modules" element={<Navigate to="modules/overview" replace />} />
              <Route path="assignments" element={<Navigate to="assignments/setup" replace />} />
              <Route
                path="assignments/config-sections"
                element={<Navigate to="assignments/config" replace />}
              />
              <Route
                path="assignments/files"
                element={<Navigate to="assignments/files/main-files" replace />}
              />
              <Route
                path="assignments/concepts"
                element={<Navigate to="assignments/tasks" replace />}
              />
              <Route
                path="assignments/submissions"
                element={<Navigate to="assignments/submissions/how-to-submit" replace />}
              />
              {/* Grading wasn't a clickable key, but support a direct path anyway */}
              <Route
                path="assignments/grading"
                element={<Navigate to="assignments/memo-output" replace />}
              />
              <Route path="support" element={<Navigate to="support/troubleshooting" replace />} />

              {/* Getting Started */}
              <Route path="getting-started/overview" element={<UnderConstruction />} />

              {/* Modules */}
              <Route path="modules/overview" element={<UnderConstruction />} />
              <Route path="modules/announcements" element={<UnderConstruction />} />
              <Route path="modules/attendance" element={<UnderConstruction />} />
              <Route path="modules/grades" element={<UnderConstruction />} />
              <Route path="modules/personnel" element={<UnderConstruction />} />

              {/* Assignments → Setup */}
              <Route path="assignments/setup" element={<AssignmentSetupHelp />} />

              {/* Assignments → Assignment Config (Overview + subsections) */}
              <Route path="assignments/config" element={<Navigate to="overview" replace />} />
              <Route path="assignments/config/overview" element={<ConfigOverviewHelp />} />
              <Route path="assignments/config/project" element={<ProjectHelp />} />
              <Route path="assignments/config/execution" element={<ExecutionHelp />} />
              <Route path="assignments/config/output" element={<OutputHelp />} />
              <Route path="assignments/config/marking" element={<MarkingHelp />} />
              <Route path="assignments/config/security" element={<SecurityHelp />} />
              <Route path="assignments/config/gatlam" element={<GATLAMHelp />} />

              {/* Assignments → Files */}
              <Route path="assignments/files/main-files" element={<MainFile />} />
              <Route path="assignments/files/makefile" element={<MakefileHelp />} />
              <Route path="assignments/files/memo-files" element={<MemoFiles />} />
              <Route path="assignments/files/specification" element={<Specification />} />

              {/* Assignments → Concepts */}
              <Route path="assignments/tasks" element={<TasksHelp />} />
              <Route path="assignments/code-coverage" element={<ConceptCodeCoverage />} />
              <Route path="assignments/gatlam" element={<ConceptGATLAM />} />

              {/* Assignments → Submissions */}
              <Route path="assignments/submissions/how-to-submit" element={<HowToSubmitHelp />} />
              <Route path="assignments/submissions/policy" element={<UnderConstruction />} />
              <Route
                path="assignments/submissions/understanding-results"
                element={<UnderConstruction />}
              />

              {/* Assignments → Grading */}
              <Route path="assignments/memo-output" element={<MemoOutputHelp />} />
              <Route path="assignments/mark-allocator" element={<MarkAllocatorHelp />} />

              {/* Support */}
              <Route path="support/troubleshooting" element={<UnderConstruction />} />
              <Route path="support/contact" element={<UnderConstruction />} />
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
