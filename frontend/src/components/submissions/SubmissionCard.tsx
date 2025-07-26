import { Card, Tag, Space, Avatar } from 'antd';
import { FileTextOutlined, UserOutlined } from '@ant-design/icons';
import dayjs from 'dayjs';
import type { ReactNode } from 'react';
import type { Submission } from '@/types/modules/assignments/submissions';

type Props = {
  submission: Submission & {
    status: 'Pending' | 'Graded';
    path: string;
    percentageMark?: number;
  };
  actions?: ReactNode[];
};

const getMarkColor = (mark: number): string => {
  if (mark >= 75) return 'green';
  if (mark >= 50) return 'orange';
  return 'red';
};

const SubmissionCard = ({ submission, actions = [] }: Props) => {
  const { user, attempt, status, is_late, percentageMark, created_at, is_practice } = submission;

  return (
    <Card
      hoverable
      actions={actions}
      className="rounded-lg border border-gray-200 dark:border-gray-700"
    >
      <Card.Meta
        avatar={user ? <Avatar icon={<UserOutlined />} /> : <Avatar icon={<FileTextOutlined />} />}
        title={
          <div className="flex justify-between items-center">
            <span className="font-semibold text-black dark:text-white">
              {user?.username ?? 'Unknown User'}
            </span>
            <Tag color="blue">Attempt #{attempt}</Tag>
          </div>
        }
        description={
          <div className="space-y-2 mt-2">
            <Space wrap>
              <Tag color={status === 'Graded' ? getMarkColor(percentageMark ?? 0) : 'default'}>
                {status === 'Graded' && typeof percentageMark === 'number'
                  ? `${percentageMark}%`
                  : 'Pending'}
              </Tag>

              <Tag color={is_late ? 'red' : 'default'}>
                {is_late ? 'Late Submission' : 'On Time'}
              </Tag>

              {is_practice && <Tag color="gold">Practice</Tag>}
            </Space>

            <div className="text-xs text-gray-500 dark:text-neutral-400">
              Submitted: {dayjs(created_at).format('YYYY-MM-DD HH:mm')}
            </div>
          </div>
        }
      />
    </Card>
  );
};

export default SubmissionCard;
