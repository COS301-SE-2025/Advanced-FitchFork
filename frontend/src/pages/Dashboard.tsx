import React, { useMemo, useState, useEffect } from 'react';
import { Card, Typography, Segmented } from 'antd';
import { useAuth } from '@/context/AuthContext';
import { useUI } from '@/context/UIContext';
import {
  AdminDashboard,
  AssistantDashboard,
  LecturerDashboard,
  StudentDashboard,
  TutorDashboard,
} from '@/components/dashboard';

const { Title, Text } = Typography;

// ---- scopes ----
type ScopeKey = 'student' | 'tutor' | 'lecturer' | 'assistant' | 'admin';
const SCOPE_STORAGE_KEY = 'dashboard:lastScope';

const RoleSummary: React.FC<{
  scope: ScopeKey;
  availableScopes: ScopeKey[];
  onScopeChange: (s: ScopeKey) => void;
}> = ({ scope, availableScopes, onScopeChange }) => {
  const { user } = useAuth();
  const { isMobile } = useUI();

  const options = availableScopes.map((s) => ({
    label:
      s === 'student'
        ? 'Student'
        : s === 'tutor'
          ? 'Tutor'
          : s === 'lecturer'
            ? 'Lecturer'
            : s === 'assistant'
              ? 'Assistant'
              : 'Admin',
    value: s,
  }));

  return (
    <Card
      className="rounded-2xl !border-gray-200 dark:!border-gray-800"
      styles={{ body: { padding: 16 } }}
    >
      <div className="flex flex-col sm:flex-row sm:items-center justify-between gap-3">
        <div className="min-w-0">
          <Title level={4} className="!mb-0 truncate">
            Welcome{user?.username ? `, ${user.username}` : ''}
          </Title>
          <Text type="secondary">This dashboard adapts to your roles across modules.</Text>
        </div>

        {availableScopes.length > 1 && (
          <div className={isMobile ? 'w-full' : ''}>
            <Segmented
              value={scope}
              onChange={(v) => onScopeChange(v as ScopeKey)}
              options={options}
              size="middle"
              block={isMobile}
              className={isMobile ? 'role-seg w-full' : 'role-seg'}
              aria-label="Role selection"
            />
          </div>
        )}
      </div>
    </Card>
  );
};

// ==================== DASHBOARD ====================
const Dashboard: React.FC = () => {
  const {
    isAdmin,
    hasLecturerRole,
    hasAssistantLecturerRole,
    hasTutorRole,
    hasStudentRole,
    modulesByRole,
  } = useAuth() as any;
  const { isLg } = useUI();

  // role flags -> available scopes
  const roleFlags = useMemo(
    () => ({
      student: hasStudentRole(),
      tutor: hasTutorRole(),
      lecturer: hasLecturerRole(),
      assistant: hasAssistantLecturerRole(),
      admin: isAdmin,
    }),
    [modulesByRole, isAdmin],
  );

  const availableScopes = useMemo<ScopeKey[]>(() => {
    const s: ScopeKey[] = [];
    if (roleFlags.student) s.push('student');
    if (roleFlags.tutor) s.push('tutor');
    if (roleFlags.lecturer) s.push('lecturer');
    if (roleFlags.assistant) s.push('assistant');
    if (roleFlags.admin) s.push('admin');
    return s;
  }, [roleFlags]);

  const [scope, setScope] = useState<ScopeKey>(() => {
    if (typeof window === 'undefined') return 'student';
    const saved = localStorage.getItem(SCOPE_STORAGE_KEY) as ScopeKey | null;
    return saved ?? 'student';
  });

  useEffect(() => {
    if (availableScopes.length && !availableScopes.includes(scope)) setScope(availableScopes[0]);
  }, [availableScopes, scope]);

  useEffect(() => {
    if (typeof window !== 'undefined') localStorage.setItem(SCOPE_STORAGE_KEY, scope);
  }, [scope]);

  const hasAnyRole =
    isAdmin ||
    hasLecturerRole() ||
    hasAssistantLecturerRole() ||
    hasTutorRole() ||
    hasStudentRole();

  return (
    <div className="h-full flex flex-col overflow-x-hidden">
      <div
        className={`flex-1 p-4 flex flex-col gap-4 ${isLg ? 'overflow-hidden' : 'overflow-y-auto mb-4'}`}
      >
        <RoleSummary scope={scope} availableScopes={availableScopes} onScopeChange={setScope} />

        {!hasAnyRole && (
          <Card className="shadow-sm rounded-2xl">
            <Text type="secondary">
              No roles assigned yet. Once you&apos;re added to a module, the dashboard will populate
              dynamically.
            </Text>
          </Card>
        )}

        <div className="flex-1 min-h-0">
          {scope === 'student' && <StudentDashboard />}
          {scope === 'tutor' && <TutorDashboard />}
          {scope === 'assistant' && <AssistantDashboard />}
          {scope === 'lecturer' && <LecturerDashboard />}
          {scope === 'admin' && <AdminDashboard />}
        </div>
      </div>
    </div>
  );
};

export default Dashboard;
