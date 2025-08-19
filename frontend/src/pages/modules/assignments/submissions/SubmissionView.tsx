import { Typography, Tag, Descriptions, Spin, Button } from 'antd';
import { DownloadOutlined } from '@ant-design/icons';

import SubmissionTasks from '@/components/submissions/SubmissionTasks';
import { useAssignment } from '@/context/AssignmentContext';
import { useModule } from '@/context/ModuleContext';
import { useEffect, useState } from 'react';
import { useParams } from 'react-router-dom';
import {
  getSubmissionDetails,
  getSubmissionOutput,
} from '@/services/modules/assignments/submissions';
import type { Submission, SubmissionTaskOutput } from '@/types/modules/assignments/submissions';
import { useBreadcrumbContext } from '@/context/BreadcrumbContext';
import { message } from '@/utils/message';
import { useAuth } from '@/context/AuthContext';
import { useViewSlot } from '@/context/ViewSlotContext';

const { Text, Title } = Typography;

const SubmissionView = () => {
  const auth = useAuth();
  const [submission, setSubmission] = useState<Submission | null>(null);
  const [submissionOutput, setSubmissionOutput] = useState<SubmissionTaskOutput[]>([]);
  const [loading, setLoading] = useState(true);
  const module = useModule();
  const { assignment, memoOutput } = useAssignment();
  const { submission_id } = useParams();
  const submissionId = Number(submission_id);
  const { setBreadcrumbLabel } = useBreadcrumbContext();
  const { setValue } = useViewSlot();

  useEffect(() => {
    setValue(
      <Typography.Text className="text-base font-medium text-gray-900 dark:text-gray-100 truncate">
        Submission {submission?.id}
      </Typography.Text>,
    );
  }, []);

  const fetchSubmission = async () => {
    if (!module.id || !assignment.id || !submission_id) return;
    setLoading(true);
    try {
      const res = await getSubmissionDetails(module.id, assignment.id, Number(submission_id));
      if (res.success && res.data) {
        setSubmission(res.data);
        setBreadcrumbLabel(
          `modules/${module.id}/assignments/${assignment.id}/submissions/${res.data.id}`,
          `Attempt #${res.data.attempt}`,
        );
      }
    } catch (err) {
      console.error('Failed to load submission details', err);
    } finally {
      setLoading(false);
    }
  };

  const fetchSubmisisonOutput = async () => {
    try {
      const res = await getSubmissionOutput(module.id, assignment.id, submissionId);
      if (res.success && res.data) {
        setSubmissionOutput(res.data);
      } else {
        message.error(res.message);
      }
    } catch {
      message.error('Failed to fetch submission output');
    }
  };

  useEffect(() => {
    fetchSubmission();
    if (!auth.isStudent(module.id)) fetchSubmisisonOutput();
  }, [module.id, assignment.id, submissionId]);

  if (loading || !submission) {
    return (
      <div className="p-6 max-w-4xl">
        <Spin />
      </div>
    );
  }

  const { mark, attempt, created_at, hash, tasks } = submission;
  const percentage = mark?.total ? Math.round((mark.earned / mark.total) * 100) : null;
  const markColor =
    percentage === null
      ? 'default'
      : percentage >= 75
        ? 'green'
        : percentage >= 50
          ? 'orange'
          : 'red';

  return (
    <div className="flex flex-col lg:flex-row gap-4 pb-4">
      {/* Left: Tasks */}
      <div className="order-2 lg:order-1 lg:w-2/3 space-y-4">
        <div className="bg-white dark:bg-gray-900 border border-gray-200 dark:border-gray-800 rounded-md p-4">
          <Title level={4} className="!mb-0">
            Tasks
          </Title>
        </div>
        <SubmissionTasks
          tasks={tasks ?? []}
          memoOutput={memoOutput}
          submisisonOutput={submissionOutput}
        />
      </div>

      {/* Right: Description */}
      <div className="order-1 lg:order-2 lg:w-1/3 space-y-6">
        <Descriptions bordered column={1} size="middle">
          {submission.user && (
            <>
              <Descriptions.Item label="Username">{submission.user.username}</Descriptions.Item>
              <Descriptions.Item label="Email">{submission.user.email}</Descriptions.Item>
            </>
          )}

          <Descriptions.Item label="Mark">
            {mark ? (
              <>
                <Tag color={markColor}>
                  {mark.earned} / {mark.total}
                </Tag>
                <Text type="secondary"> ({percentage}%)</Text>
              </>
            ) : (
              <Tag color="default">Not graded</Tag>
            )}
          </Descriptions.Item>

          <Descriptions.Item label="Attempt">
            <Tag>#{attempt}</Tag>
          </Descriptions.Item>

          <Descriptions.Item label="Uploaded At">
            {created_at ? new Date(created_at).toLocaleString() : '—'}
          </Descriptions.Item>

          <Descriptions.Item label="File Hash (MD5)">
            <Text code>{hash || '—'}</Text>
          </Descriptions.Item>

          <Descriptions.Item label="File">
            <Button icon={<DownloadOutlined />} type="link" size="small">
              Download File
            </Button>
          </Descriptions.Item>
        </Descriptions>
      </div>
    </div>
  );
};

export default SubmissionView;
