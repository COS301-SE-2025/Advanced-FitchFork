import { AssignmentProvider } from '@/context/AssignmentContext';
import { useBreadcrumbContext } from '@/context/BreadcrumbContext';
import { useModule } from '@/context/ModuleContext';
import { getAssignmentDetails, getAssignmentReadiness } from '@/services/modules/assignments';
import type { Assignment, AssignmentFile, AssignmentReadiness } from '@/types/modules/assignments';
import { Alert, Spin } from 'antd';
import { useEffect, useState } from 'react';
import { Outlet, useParams } from 'react-router-dom';

interface AssignmentDetails extends Assignment {
  files: AssignmentFile[];
}

const SubmissionLayout = () => {
  const module = useModule();
  const { assignment_id } = useParams();
  const { setBreadcrumbLabel } = useBreadcrumbContext();

  const [assignment, setAssignment] = useState<AssignmentDetails | null>(null);
  const [readiness, setReadiness] = useState<AssignmentReadiness | null>(null);
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
    <AssignmentProvider value={{ assignment, readiness, refreshReadiness }}>
      <Outlet />
    </AssignmentProvider>
  );
};

export default SubmissionLayout;
