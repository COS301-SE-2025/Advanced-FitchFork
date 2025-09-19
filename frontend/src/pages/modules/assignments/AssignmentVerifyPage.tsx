import { useEffect, useState } from 'react';
import { useNavigate, useParams } from 'react-router-dom';
import { Button, Card, Form, Input, Typography, Alert } from 'antd';
import { useAuth } from '@/context/AuthContext';
import { useModule } from '@/context/ModuleContext';
import { useBreadcrumbContext } from '@/context/BreadcrumbContext';
import { useViewSlot } from '@/context/ViewSlotContext';
import { listAssignments, verifyAssignment } from '@/services/modules/assignments';
import { setAssignmentPin } from '@/utils/assignmentAccess';

export default function AssignmentVerifyPage() {
  const { id: moduleIdParam, assignment_id: assignmentIdParam } = useParams();
  const moduleId = Number(moduleIdParam);
  const assignmentId = Number(assignmentIdParam);

  const navigate = useNavigate();
  const module = useModule();
  const { isAdmin, isLecturer, isAssistantLecturer, isTutor } = useAuth();
  const { setBreadcrumbLabel } = useBreadcrumbContext();
  const { setValue, setBackTo } = useViewSlot();

  const assignmentPath = `/modules/${moduleId}/assignments/${assignmentId}`;
  const listPath = `/modules/${moduleId}/assignments`;

  const [assignmentName, setAssignmentName] = useState<string>('Assignment');
  const [submitting, setSubmitting] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [form] = Form.useForm<{ pin: string }>();

  // Staff never need to verify — always go straight to the assignment page
  const isStaffLike =
    isAdmin || isLecturer(moduleId) || isAssistantLecturer(moduleId) || isTutor(moduleId);
  useEffect(() => {
    if (isStaffLike) navigate(assignmentPath, { replace: true });
  }, [isStaffLike, navigate, assignmentPath]);

  // Fetch name for nicer UI (safe endpoint)
  useEffect(() => {
    (async () => {
      try {
        const res = await listAssignments(moduleId, { page: 1, per_page: 200 });
        if (res.success && res.data?.assignments) {
          const a = res.data.assignments.find((a) => a.id === assignmentId);
          if (a) setAssignmentName(a.name);
        }
      } catch {
        /* ignore */
      }
    })();
  }, [moduleId, assignmentId]);

  // Breadcrumbs
  useEffect(() => {
    if (!Number.isFinite(moduleId) || !Number.isFinite(assignmentId)) return;
    setBreadcrumbLabel(
      `modules/${moduleId}/assignments/${assignmentId}`,
      assignmentName || String(assignmentId),
    );
    setBreadcrumbLabel(`modules/${moduleId}/assignments/${assignmentId}/verify`, 'Verify access');
  }, [moduleId, assignmentId, assignmentName]);

  // Mobile header slot
  useEffect(() => {
    setValue(
      <Typography.Text className="text-base font-medium text-gray-900 dark:text-gray-100 truncate">
        Verify access
      </Typography.Text>,
    );
    setBackTo(listPath);
    return () => {
      setValue(null);
      setBackTo(null);
    };
  }, [setValue, setBackTo, listPath]);

  const onSubmit = async () => {
    setError(null);
    const { pin } = await form.validateFields();
    setSubmitting(true);
    try {
      const res = await verifyAssignment(moduleId, assignmentId, pin);
      if (!res.success) {
        setError(res.message ?? 'Verification failed');
        return;
      }
      setAssignmentPin(moduleId, assignmentId, pin);
      navigate(assignmentPath, { replace: true });
    } catch (e: any) {
      setError(e?.message ?? 'Verification failed');
    } finally {
      setSubmitting(false);
    }
  };

  return (
    <div className="max-w-md mx-auto mt-10 px-3">
      <Card>
        <Typography.Title level={4} className="!mt-0 !mb-2">
          Verify Access
        </Typography.Title>
        <Typography.Paragraph type="secondary" className="!mb-4">
          {module.code} • {module.year} — {assignmentName}
        </Typography.Paragraph>

        {error && <Alert type="error" message={error} showIcon className="!mb-4" />}

        <Form form={form} layout="vertical" onFinish={onSubmit}>
          <Form.Item
            label="Assignment PIN"
            name="pin"
            rules={[{ required: true, message: 'Please enter the PIN' }]}
          >
            <Input autoFocus placeholder="Enter PIN" />
          </Form.Item>

          <div className="!space-y-2">
            <Button type="primary" htmlType="submit" loading={submitting} block>
              Verify
            </Button>
            <Button block onClick={() => navigate(listPath, { replace: true })}>
              Back to Assignments
            </Button>
          </div>
        </Form>
      </Card>
    </div>
  );
}
