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
import TasksLayout from './layouts/TasksLayout';
import TasksIndex from './pages/modules/assignments/tasks/TasksIndex';
import AssignmentFiles from './pages/modules/assignments/AssignmentFiles';
import MemoOutput from './pages/modules/assignments/MemoOutput';
import MarkAllocator from './pages/modules/assignments/MarkAllocator';
import AssignmentStepUpload from './pages/modules/assignments/steps/AssignmentStepUpload';
import AssignmentSteps from './pages/modules/assignments/steps/AssignmentSteps';
import HelpPageLayout from './layouts/HelpPageLayout';
import HelpAccount from './pages/help/HelpAccount';
import HelpAssignments from './pages/help/HelpAssignments';
import HelpContact from './pages/help/HelpContact';
import HelpSubmissions from './pages/help/HelpSubmissions';
import HelpTroubleshooting from './pages/help/HelpTroubleshooting';
import AssignmentLayoutWrapper from './layouts/AssignmentLayoutWrapper';
import ConfigStep from './pages/modules/assignments/steps/ConfigStep';
import TaskStep from './pages/modules/assignments/steps/TaskStep';
import GenerateMemoOutputStep from './pages/modules/assignments/steps/GenerateMemoOutputStep';
import GenerateMarkAllocatorStep from './pages/modules/assignments/steps/GenerateMarkAllocatorStep';
import Config from './pages/modules/assignments/Config';

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

            {/* STEPS: only declared here to avoid duplicate renders */}
            <Route
              path="assignments/:assignment_id/steps"
              element={
                <AssignmentLayoutWrapper>
                  <AssignmentSteps />
                </AssignmentLayoutWrapper>
              }
            >
              <Route path="config" element={<ConfigStep />} />
              <Route path="main" element={<AssignmentStepUpload fileType="main" />} />
              <Route path="memo" element={<AssignmentStepUpload fileType="memo" />} />
              <Route path="makefile" element={<AssignmentStepUpload fileType="makefile" />} />
              {/* <Route path="tasks" element={<TasksLayout />}>
                <Route index element={<TasksIndex />} />
                <Route path=":task_id" element={<UnderConstruction />} />
              </Route> */}
              <Route path="tasks" element={<TaskStep />}></Route>
              <Route path="tasks/:task_id" element={<TaskStep />} />
              <Route path="memo-output" element={<GenerateMemoOutputStep />} />
              <Route path="mark-allocator" element={<GenerateMarkAllocatorStep />} />
            </Route>

            <Route path="assignments" element={<Assignments />} />
            <Route path="assignments/:assignment_id" element={<AssignmentLayout />}>
              <Route index element={<Navigate to="submissions" replace />} />
              <Route path="files" element={<AssignmentFiles />} />
              <Route path="submissions" element={<Submissions />} />
              <Route path="tasks" element={<TasksLayout />}>
                <Route index element={<TasksIndex />} />
                <Route path=":task_id" element={<UnderConstruction />} />
              </Route>
              <Route path="memo-output" element={<MemoOutput />} />
              <Route path="mark-allocator" element={<MarkAllocator />} />
              <Route path="stats" element={<UnderConstruction />} />
              <Route path="config" element={<Config />} />
            </Route>

            <Route path="assignments/:assignment_id" element={<SubmissionLayout />}>
              <Route path="submissions/:submission_id" element={<SubmissionView />} />
            </Route>

            <Route path="grades" element={<UnderConstruction />} />
            <Route path="resources" element={<UnderConstruction />} />
            <Route path="personnel" element={<ModulePersonnel />} />
          </Route>

          {/* Fallbacks */}
          <Route path="/modules/:module_id/assignments" element={<Unauthorized />} />
          <Route path="/modules/:module_id/assignments/:assignment_id" element={<Unauthorized />} />

          <Route path="/reports" element={<UnderConstruction />} />

          <Route path="/help" element={<HelpPageLayout />}>
            <Route path="account" element={<HelpAccount />} />
            <Route path="assignments" element={<HelpAssignments />} />
            <Route path="submissions" element={<HelpSubmissions />} />
            <Route path="troubleshooting" element={<HelpTroubleshooting />} />
            <Route path="contact" element={<HelpContact />} />
            <Route index element={<Navigate to="account" replace />} />
          </Route>
        </Route>

        <Route path="*" element={<NotFound />} />
      </Routes>
    </Router>
  );
}
