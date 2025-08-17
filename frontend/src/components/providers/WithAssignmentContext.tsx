import { useEffect, useState } from 'react';
import { useParams, Outlet } from 'react-router-dom';
import { Spin } from 'antd';

import { useModule } from '@/context/ModuleContext';
import { AssignmentProvider } from '@/context/AssignmentContext';
import { useBreadcrumbContext } from '@/context/BreadcrumbContext';

import { getAssignmentDetails, getAssignmentReadiness } from '@/services/modules/assignments';
import { getMemoOutput } from '@/services/modules/assignments/memo-output';
import { getMarkAllocator } from '@/services/modules/assignments/mark-allocator';

import type { Assignment, AssignmentFile, AssignmentReadiness } from '@/types/modules/assignments';
import type { MemoTaskOutput } from '@/types/modules/assignments/memo-output';
import type { MarkAllocatorFile } from '@/types/modules/assignments/mark-allocator';

export interface AssignmentDetails extends Assignment {
  files: AssignmentFile[];
}

export default function WithAssignmentContext() {
  const module = useModule();
  const { assignment_id } = useParams();
  const assignmentIdNum = Number(assignment_id);
  const { setBreadcrumbLabel } = useBreadcrumbContext();

  const [assignment, setAssignment] = useState<AssignmentDetails | null>(null);
  const [memoOutput, setMemoOutput] = useState<MemoTaskOutput[]>([]);
  const [markAllocator, setMarkAllocator] = useState<MarkAllocatorFile | null>(null);
  const [readiness, setReadiness] = useState<AssignmentReadiness | null>(null);
  const [loading, setLoading] = useState(true);

  const refreshAssignment = async () => {
    setLoading(true);
    const [detailsRes, readinessRes, memoRes, allocatorRes] = await Promise.all([
      getAssignmentDetails(module.id, assignmentIdNum),
      getAssignmentReadiness(module.id, assignmentIdNum),
      getMemoOutput(module.id, assignmentIdNum),
      getMarkAllocator(module.id, assignmentIdNum),
    ]);

    if (detailsRes.success && detailsRes.data) {
      setAssignment(detailsRes.data);
      setBreadcrumbLabel(
        `modules/${module.id}/assignments/${detailsRes.data.id}`,
        detailsRes.data.name,
      );
    }

    if (readinessRes.success) setReadiness(readinessRes.data);
    if (memoRes.success && memoRes.data) setMemoOutput(memoRes.data);
    if (allocatorRes.success && allocatorRes.data) setMarkAllocator(allocatorRes.data);

    setLoading(false);
  };

  useEffect(() => {
    if (!isNaN(assignmentIdNum)) {
      refreshAssignment();
    } else {
      setLoading(false);
    }
  }, [assignmentIdNum, module.id]);

  if (loading || !assignment) {
    return <Spin className="p-6" tip="Loading assignment..." />;
  }

  return (
    <AssignmentProvider
      value={{
        assignment,
        memoOutput,
        markAllocator,
        readiness,
        refreshAssignment,
        loading,
      }}
    >
      <Outlet />
    </AssignmentProvider>
  );
}
