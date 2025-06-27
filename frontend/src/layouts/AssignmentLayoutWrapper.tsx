import { useEffect, useState } from 'react';
import { useParams } from 'react-router-dom';
import { useModule } from '@/context/ModuleContext';
import { AssignmentProvider } from '@/context/AssignmentContext';
import { getAssignmentDetails, getAssignmentReadiness } from '@/services/modules/assignments';
import type { Assignment, AssignmentFile, AssignmentReadiness } from '@/types/modules/assignments';
import { Spin, Alert } from 'antd';

interface AssignmentDetails extends Assignment {
  files: AssignmentFile[];
}

const AssignmentLayoutWrapper = ({ children }: { children: React.ReactNode }) => {
  const module = useModule();
  const { assignment_id } = useParams();
  const assignmentIdNum = Number(assignment_id);

  const [assignment, setAssignment] = useState<AssignmentDetails | null>(null);
  const [readiness, setReadiness] = useState<AssignmentReadiness | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  const load = async () => {
    setLoading(true);
    const res1 = await getAssignmentDetails(module.id, assignmentIdNum);
    const res2 = await getAssignmentReadiness(module.id, assignmentIdNum);
    if (res1.success && res1.data && res2.success) {
      setAssignment(res1.data);
      setReadiness(res2.data);
    } else {
      setError(res1.message || res2.message || 'Failed to load data');
    }
    setLoading(false);
  };

  useEffect(() => {
    if (!isNaN(assignmentIdNum)) load();
    else {
      setError('Invalid assignment ID');
      setLoading(false);
    }
  }, [assignmentIdNum]);

  const refreshReadiness = async () => {
    const res = await getAssignmentReadiness(module.id, assignmentIdNum);
    if (res.success) setReadiness(res.data);
  };

  if (loading) return <Spin className="p-6" tip="Loading..." />;
  if (error || !assignment) return <Alert className="p-6" type="error" message={error} />;

  return (
    <AssignmentProvider value={{ assignment, readiness, refreshReadiness }}>
      {children}
    </AssignmentProvider>
  );
};

export default AssignmentLayoutWrapper;
