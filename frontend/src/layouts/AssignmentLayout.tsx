import { useEffect, useState } from 'react';
import { useNavigate, useLocation, Outlet, useParams } from 'react-router-dom';
import { Tabs, Spin, Alert, message, Button } from 'antd';
import { useModule } from '@/context/ModuleContext';
import { useAuth } from '@/context/AuthContext';
import { AssignmentProvider } from '@/context/AssignmentContext';
import { useBreadcrumbContext } from '@/context/BreadcrumbContext';
import PageHeader from '@/components/PageHeader';
import { getAssignmentDetails, getAssignmentReadiness } from '@/services/modules/assignments';
import { generateMemoOutput } from '@/services/modules/assignments/memo-output';
import { generateMarkAllocator } from '@/services/modules/assignments/mark-allocator/post';
import type { Assignment, AssignmentFile, AssignmentReadiness } from '@/types/modules/assignments';

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
  const [error, setError] = useState<string | null>(null);
  const [readiness, setReadiness] = useState<AssignmentReadiness | null>(null);

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
        setError(null);
        setBreadcrumbLabel(`modules/${module.id}/assignments/${res.data.id}`, res.data.name);
      } else {
        setError(res.message || 'Failed to load assignment.');
      }
      setLoading(false);
    };

    const loadReadiness = async () => {
      const res = await getAssignmentReadiness(module.id, assignmentIdNum);
      if (res.success) {
        setReadiness(res.data);
      }
    };

    if (!isNaN(assignmentIdNum)) {
      loadAssignment();
      loadReadiness();
    } else {
      setError('Invalid assignment ID');
      setLoading(false);
    }
  }, [module.id, assignmentIdNum]);

  const refreshReadiness = async () => {
    const res = await getAssignmentReadiness(module.id, assignmentIdNum);
    if (res.success) {
      setReadiness(res.data);
    }
  };

  if (loading) {
    return (
      <div className="p-6">
        <Spin tip="Loading assignment..." />
      </div>
    );
  }

  if (error || !assignment) {
    return (
      <div className="p-6">
        <Alert type="error" message="Error" description={error} showIcon />
      </div>
    );
  }

  return (
    <div className="p-4 sm:p-6 space-y-6">
      <PageHeader
        title={assignment.name}
        description={`Manage assignment #${assignment.id} in ${module.code}`}
        extra={
          <div className="flex gap-2 flex-wrap">
            <Button
              onClick={async () => {
                const res = await generateMemoOutput(module.id, assignment.id);
                if (res.success) {
                  message.success('Memo output generated successfully.');
                  await refreshReadiness();
                } else {
                  message.error(`Failed to generate memo: ${res.message}`);
                }
              }}
            >
              Generate Memo Output
            </Button>
            <Button
              onClick={async () => {
                const res = await generateMarkAllocator(module.id, assignment.id);
                if (res.success) {
                  message.success('Mark allocator generated successfully.');
                  await refreshReadiness();
                } else {
                  message.error(`Failed to generate mark allocator: ${res.message}`);
                }
              }}
            >
              Generate Mark Allocator
            </Button>
          </div>
        }
      />

      {showTabs && (
        <Tabs
          activeKey={activeKey}
          onChange={(key) => navigate(key)}
          items={tabs}
          tabBarGutter={16}
          className="!mb-4"
        />
      )}

      <AssignmentProvider value={{ assignment, refreshReadiness, readiness }}>
        <Outlet />
      </AssignmentProvider>
    </div>
  );
};

export default AssignmentLayout;
