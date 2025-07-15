import { useEffect, useState } from 'react';
import { useNavigate, useLocation, Outlet, useParams } from 'react-router-dom';
import { Tabs, Spin, Button } from 'antd';
import { useModule } from '@/context/ModuleContext';
import { useAuth } from '@/context/AuthContext';
import { AssignmentProvider } from '@/context/AssignmentContext';
import { useBreadcrumbContext } from '@/context/BreadcrumbContext';
import PageHeader from '@/components/PageHeader';
import { getAssignmentDetails, getAssignmentReadiness } from '@/services/modules/assignments';
import type { Assignment, AssignmentFile, AssignmentReadiness } from '@/types/modules/assignments';
import AssignmentSetup from '@/pages/modules/assignments/steps/AssignmentSetup';

interface AssignmentDetails extends Assignment {
  files: AssignmentFile[];
}

const AssignmentLayout = () => {
  const module = useModule();
  const { isStudent, isLecturer, isAdmin } = useAuth();
  const { assignment_id } = useParams();
  const navigate = useNavigate();
  const location = useLocation();
  const { setBreadcrumbLabel } = useBreadcrumbContext();

  const [assignment, setAssignment] = useState<AssignmentDetails | null>(null);
  const [loading, setLoading] = useState(true);
  const [readiness, setReadiness] = useState<AssignmentReadiness | null>(null);
  const [setupOpen, setSetupOpen] = useState(false);

  const assignmentIdNum = Number(assignment_id);
  const basePath = `/modules/${module.id}/assignments/${assignment_id}`;
  const showTabs = !isStudent(module.id) || isAdmin;

  const tabs = [
    { key: `${basePath}/submissions`, label: 'Submissions', disabled: false },
    ...(isLecturer(module.id) || isAdmin
      ? [
          { key: `${basePath}/files`, label: 'Files', disabled: !readiness?.config_present },
          { key: `${basePath}/tasks`, label: 'Tasks', disabled: !readiness?.config_present },
          {
            key: `${basePath}/memo-output`,
            label: 'Memo Output',
            disabled: !readiness?.config_present || !readiness?.tasks_present,
          },
          {
            key: `${basePath}/mark-allocator`,
            label: 'Mark Allocator',
            disabled: !readiness?.memo_output_present,
          },
          { key: `${basePath}/config`, label: 'Config', disabled: false },
          {
            key: `${basePath}/stats`,
            label: 'Statistics',
            disabled: !readiness?.is_ready,
          },
        ]
      : []),
  ];

  const activeKey =
    tabs.find((tab) => location.pathname === tab.key || location.pathname.startsWith(tab.key + '/'))
      ?.key || `${basePath}/submissions`;

  useEffect(() => {
    const loadAssignment = async () => {
      setLoading(true);
      const res = await getAssignmentDetails(module.id, assignmentIdNum);
      if (res.success && res.data) {
        setAssignment(res.data);
        setBreadcrumbLabel(`modules/${module.id}/assignments/${res.data.id}`, res.data.name);
      }
      setLoading(false);
    };

    const loadReadiness = async () => {
      const res = await getAssignmentReadiness(module.id, assignmentIdNum);
      if (res.success) setReadiness(res.data);
    };

    if (!isNaN(assignmentIdNum)) {
      loadAssignment();
      loadReadiness();
    } else {
      setLoading(false);
    }
  }, [module.id, assignmentIdNum]);

  const refreshReadiness = async () => {
    const res = await getAssignmentReadiness(module.id, assignmentIdNum);
    if (res.success) setReadiness(res.data);
  };

  if (loading || !assignment) {
    return <Spin className="p-6" tip="Loading assignment..." />;
  }

  return (
    <div className="p-4 sm:p-6 space-y-6">
      <div className="flex justify-between flex-wrap gap-4 m-0">
        <PageHeader
          title={assignment.name}
          description={`Manage assignment #${assignment.id} in ${module.code}`}
        />

        {readiness && !readiness.is_ready && (
          <Button type="primary" onClick={() => setSetupOpen(true)}>
            Complete Setup
          </Button>
        )}
      </div>

      <Tabs
        activeKey={activeKey}
        onChange={(key) => navigate(key)}
        items={showTabs ? tabs : []}
        tabBarGutter={16}
        className="!mb-4"
      />

      <AssignmentProvider value={{ assignment, refreshReadiness, readiness }}>
        <Outlet />
      </AssignmentProvider>

      <AssignmentSetup
        open={setupOpen}
        onClose={() => setSetupOpen(false)}
        assignmentId={assignment.id}
        module={module}
      />
    </div>
  );
};

export default AssignmentLayout;
