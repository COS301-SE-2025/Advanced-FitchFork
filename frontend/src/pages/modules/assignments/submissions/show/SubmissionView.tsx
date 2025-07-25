import { Typography, Tag, Descriptions, Tabs, Progress, Card, Spin, Button } from 'antd';
import { DownloadOutlined, CheckCircleTwoTone, WarningTwoTone } from '@ant-design/icons';

import SubmissionTasks from '@/components/submissions/SubmisisonTasks';
import { useAssignment } from '@/context/AssignmentContext';
import { useModule } from '@/context/ModuleContext';
import { useEffect, useState } from 'react';
import { useParams } from 'react-router-dom';
import { getSubmissionDetails } from '@/services/modules/assignments/submissions';
import type { Submission } from '@/types/modules/assignments/submissions';
import { useBreadcrumbContext } from '@/context/BreadcrumbContext';

const { Title, Text, Paragraph } = Typography;

const SubmissionView = () => {
  const [submission, setSubmission] = useState<Submission | null>(null);
  const [loading, setLoading] = useState(true);
  const module = useModule();
  const { assignment } = useAssignment();
  const { submission_id } = useParams();
  const { setBreadcrumbLabel } = useBreadcrumbContext();

  useEffect(() => {
    const fetch = async () => {
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
    fetch();
  }, [module.id, assignment.id, submission_id]);

  if (loading || !submission) {
    return (
      <div className="p-6 max-w-4xl">
        <Spin />
      </div>
    );
  }

  const { mark, attempt, created_at, hash, tasks, code_coverage } = submission;
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
    <div className="bg-white dark:bg-gray-950 border border-gray-200 dark:border-gray-700 rounded-sm p-4 xl:col-span-3 space-y-6">
      <div className="flex justify-between items-center mb-2">
        <Title level={4} className="mb-0">
          Attempt #{attempt}
        </Title>
      </div>

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

      <Tabs
        defaultActiveKey="tasks"
        size="large"
        items={[
          {
            key: 'tasks',
            label: 'Tasks',
            children: <SubmissionTasks tasks={tasks ?? []} />,
          },
          ...(code_coverage && code_coverage.length > 0
            ? [
                {
                  key: 'testing',
                  label: 'Testing',
                  children: (
                    <Card
                      title="Code Coverage"
                      className="!border-gray-200 dark:!border-neutral-800 dark:!bg-neutral-900"
                    >
                      <div className="grid gap-4 sm:grid-cols-2">
                        {code_coverage.map(({ class: cls, percentage }) => {
                          const color = percentage === 100 ? undefined : '#faad14';
                          const status = percentage === 100 ? 'success' : 'active';
                          return (
                            <div
                              key={cls}
                              className="p-4 rounded-lg border border-gray-100 dark:border-neutral-700 bg-white dark:bg-neutral-800"
                            >
                              <div className="flex justify-between items-center mb-2">
                                <Text className="font-medium">{cls}</Text>
                                <Text type="secondary">{percentage.toFixed(2)}%</Text>
                              </div>
                              <Progress percent={percentage} status={status} strokeColor={color} />
                            </div>
                          );
                        })}
                      </div>
                    </Card>
                  ),
                },
              ]
            : []),
          {
            key: 'feedback',
            label: 'Feedback',
            children: (
              <Card title="Evaluator's Feedback" bordered>
                <Paragraph>
                  <CheckCircleTwoTone twoToneColor="#52c41a" className="mr-2" />
                  This submission is well structured.
                </Paragraph>
                <Paragraph>
                  <WarningTwoTone twoToneColor="#faad14" className="mr-2" />
                  The result seems slightly off — double-check your output.
                </Paragraph>
              </Card>
            ),
          },
        ]}
      />
    </div>
  );
};

export default SubmissionView;
