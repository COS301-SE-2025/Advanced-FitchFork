import { useEffect, useState } from 'react';
import { useNavigate, useLocation, Outlet, useParams } from 'react-router-dom';
import { Tabs, Spin, Alert } from 'antd';

import { useModule } from '@/context/ModuleContext';
import { useAuth } from '@/context/AuthContext';
import { AssignmentProvider } from '@/context/AssignmentContext';
import { useBreadcrumbContext } from '@/context/BreadcrumbContext';
import PageHeader from '@/components/PageHeader';
import { getAssignmentDetails } from '@/services/modules/assignments';
import type { Assignment, AssignmentFile } from '@/types/modules/assignments';

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

  const assignmentIdNum = Number(assignment_id);
  const basePath = `/modules/${module.id}/assignments/${assignment_id}`;

  const showTabs = !isStudent(module.id) || isAdmin; // hide all tabs if student

  const tabs = [
    { key: `${basePath}/submissions`, label: 'Submissions' },
    ...(isLecturer(module.id) || isAdmin
      ? [
          { key: `${basePath}/tasks`, label: 'Tasks' },
          { key: `${basePath}/config`, label: 'Config' },
          { key: `${basePath}/stats`, label: 'Statistics' },
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
        const breadcrumbKey = `modules/${module.id}/assignments/${res.data.id}`;
        setBreadcrumbLabel(breadcrumbKey, res.data.name);
      } else {
        setError(res.message || 'Failed to load assignment.');
      }
      setLoading(false);
    };

    if (!isNaN(assignmentIdNum)) {
      loadAssignment();
    } else {
      setError('Invalid assignment ID');
      setLoading(false);
    }
  }, [module.id, assignmentIdNum]);

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

      <AssignmentProvider value={{ assignment }}>
        <Outlet />
      </AssignmentProvider>
    </div>
  );
};

export default AssignmentLayout;
