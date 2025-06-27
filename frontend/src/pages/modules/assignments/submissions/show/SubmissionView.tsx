import {
  Typography,
  Tag,
  Button,
  Descriptions,
  Tabs,
  Progress,
  Alert,
  Card,
  Space,
  Spin,
} from 'antd';
import {
  DownloadOutlined,
  FileDoneOutlined,
  CheckCircleTwoTone,
  WarningTwoTone,
} from '@ant-design/icons';
import SubmissionTasks from '@/components/submissions/SubmisisonTasks';
import { useAssignment } from '@/context/AssignmentContext';
import { useModule } from '@/context/ModuleContext';
import { useEffect, useState } from 'react';
import { useParams } from 'react-router-dom';
import { getSubmissionDetails } from '@/services/modules/assignments/submissions';
import type { Submission } from '@/types/modules/assignments/submissions';

const { Title, Text, Paragraph } = Typography;

// const steps = [
//   { title: 'Info', description: 'Submission Details' },
//   { title: 'Tasks', description: 'Breakdown of all sections' },
//   {
//     title: 'Testing',
//     description: 'Code coverage results',
//     icon: <LoadingOutlined spin />,
//   },
//   { title: 'Feedback', description: 'Evaluator remarks' },
// ];

const SubmissionView = () => {
  const [submission, setSubmission] = useState<Submission | null>(null);
  const [loading, setLoading] = useState(true);
  const module = useModule();
  const { assignment } = useAssignment();
  const { submission_id } = useParams();

  useEffect(() => {
    const fetch = async () => {
      if (!module.id || !assignment.id || !submission_id) return;
      setLoading(true);
      try {
        const res = await getSubmissionDetails(module.id, assignment.id, Number(submission_id));
        if (res.success && res.data) {
          setSubmission(res.data);
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
    <div className="p-4 sm:p-6 max-w-7xl">
      <div className="grid grid-cols-1 xl:grid-cols-4 gap-6">
        <div className="xl:col-span-3 space-y-6">
          <div className="flex justify-between items-center flex-wrap gap-4">
            <div>
              <Title level={4} className="mb-0">
                {assignment.name} – {module.code}
              </Title>
              <Text type="secondary">Marking Scheme: Best mark</Text>
            </div>
            <Space wrap>
              <Button type="primary" icon={<FileDoneOutlined />}>
                New Practice Submission
              </Button>
              <Button icon={<DownloadOutlined />}>Download</Button>
            </Space>
          </div>

          {assignment.due_date && new Date() > new Date(assignment.due_date) && (
            <Alert
              message="Past Due Date - Practice submissions only"
              description="Practice submissions won't be considered for your final mark."
              type="warning"
              showIcon
              className="!mb-4"
            />
          )}

          <Descriptions title="Submission Info" bordered column={2} size="middle">
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
          </Descriptions>

          <Tabs defaultActiveKey="tasks" size="large">
            <Tabs.TabPane tab="Tasks" key="tasks">
              <SubmissionTasks tasks={tasks ?? []} />
            </Tabs.TabPane>

            {code_coverage && code_coverage.length > 0 && (
              <Tabs.TabPane tab="Testing" key="testing">
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
              </Tabs.TabPane>
            )}

            <Tabs.TabPane tab="Feedback" key="feedback">
              <Card title="Evaluator's Feedback" bordered>
                <Paragraph>
                  <CheckCircleTwoTone twoToneColor="#52c41a" className="mr-2" />
                  This submission is well structured and passes all tests.
                </Paragraph>
                <Paragraph>
                  <WarningTwoTone twoToneColor="#faad14" className="mr-2" />
                  Consider optimizing product purchase logic in Section 2.
                </Paragraph>
              </Card>
            </Tabs.TabPane>
          </Tabs>
        </div>

        <div className="hidden xl:block">
          {/* <Steps direction="vertical" current={1} items={steps} /> */}
        </div>
      </div>
    </div>
  );
};

export default SubmissionView;
