import { AssignmentProvider } from '@/context/AssignmentContext';
import { useBreadcrumbContext } from '@/context/BreadcrumbContext';
import { useModule } from '@/context/ModuleContext';
import { getAssignmentDetails } from '@/services/modules/assignments';
import type { Assignment, AssignmentFile } from '@/types/modules/assignments';
import { Alert, Spin } from 'antd';
import { useEffect, useState } from 'react';
import { Outlet, useParams } from 'react-router-dom';

interface AssignmentDetails extends Assignment {
  files: AssignmentFile[];
}

// TODO Add submission provider for context of submission for nested routes
const SubmissionLayout = () => {
  const module = useModule();
  const { assignment_id } = useParams();
  const { setBreadcrumbLabel } = useBreadcrumbContext();

  const [assignment, setAssignment] = useState<AssignmentDetails | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  const assignmentIdNum = Number(assignment_id);

  useEffect(() => {
    const loadAssignment = async () => {
      setLoading(true);
      const res = await getAssignmentDetails(module.id, assignmentIdNum);
      if (res.success && res.data) {
        setAssignment(res.data);
        setError(null);
        console.log(res);
        const breadcrumbKey = `modules/${module.id}/assignments/${res.data.id}`;
        console.log(breadcrumbKey);
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
    <AssignmentProvider value={{ assignment }}>
      <Outlet />
    </AssignmentProvider>
  );
};

export default SubmissionLayout;
