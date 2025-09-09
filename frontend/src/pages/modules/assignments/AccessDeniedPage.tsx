import { useEffect, useMemo, useState } from 'react';
import { useLocation, useNavigate, useParams } from 'react-router-dom';
import { Alert, Button, Card, Typography } from 'antd';

import { useModule } from '@/context/ModuleContext';
import { useBreadcrumbContext } from '@/context/BreadcrumbContext';
import { useViewSlot } from '@/context/ViewSlotContext';
import { listAssignments } from '@/services/modules/assignments';

export default function AccessDeniedPage() {
  const { id: moduleIdParam, assignment_id: assignmentIdParam } = useParams();
  const moduleId = Number(moduleIdParam);
  const assignmentId = Number(assignmentIdParam);

  const module = useModule();
  const navigate = useNavigate();
  const location = useLocation();
  const { setBreadcrumbLabel } = useBreadcrumbContext();
  const { setValue, setBackTo } = useViewSlot();

  const messageFromState = (location.state as any)?.message as string | undefined;

  const [assignmentName, setAssignmentName] = useState<string | null>(null);

  useEffect(() => {
    if (!Number.isFinite(moduleId) || !Number.isFinite(assignmentId)) return;
    (async () => {
      try {
        const res = await listAssignments(moduleId, { page: 1, per_page: 200 });
        const found = res.success ? res.data?.assignments.find((a) => a.id === assignmentId) : null;
        if (found) setAssignmentName(found.name);
      } catch {
        /* ignore */
      }
    })();
  }, [moduleId, assignmentId]);

  useEffect(() => {
    if (!Number.isFinite(moduleId) || !Number.isFinite(assignmentId)) return;
    setBreadcrumbLabel(
      `modules/${moduleId}/assignments/${assignmentId}`,
      assignmentName ? assignmentName : String(assignmentId),
    );
    setBreadcrumbLabel(
      `modules/${moduleId}/assignments/${assignmentId}/access-denied`,
      'Access denied',
    );
  }, [moduleId, assignmentId, assignmentName]);

  const listPath = `/modules/${moduleId}/assignments`;
  const assignmentPath = `/modules/${moduleId}/assignments/${assignmentId}`;

  useEffect(() => {
    setValue(
      <Typography.Text className="text-base font-medium text-gray-900 dark:text-gray-100 truncate">
        Access denied
      </Typography.Text>,
    );
    setBackTo(listPath);
    return () => {
      setValue(null);
      setBackTo(null);
    };
  }, [setValue, setBackTo, listPath]);

  const subtitle = useMemo(
    () =>
      `${module.code} • ${module.year} — ${
        assignmentName ? assignmentName : `Assignment ${assignmentIdParam}`
      }`,
    [module, assignmentName, assignmentIdParam],
  );

  return (
    <div className="max-w-lg mx-auto mt-10 px-3">
      <Card>
        <Typography.Title level={4} className="!mt-0">
          Access denied
        </Typography.Title>
        <Typography.Paragraph type="secondary" className="!mb-4">
          {subtitle}
        </Typography.Paragraph>

        <Alert
          type="error"
          showIcon
          className="!mb-4"
          message={messageFromState || "You don't have permission to access this assignment."}
          description={
            <Typography.Text>
              If you think this is a mistake, please contact a staff member.
            </Typography.Text>
          }
        />

        <div className="!space-y-2">
          <Button type="primary" block onClick={() => navigate(assignmentPath, { replace: true })}>
            Try Again
          </Button>
          <Button block onClick={() => navigate(listPath, { replace: true })}>
            Back to Assignments
          </Button>
        </div>
      </Card>
    </div>
  );
}
