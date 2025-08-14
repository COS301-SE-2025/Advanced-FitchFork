import { List, Avatar, Tag, Space } from 'antd';
import { FileTextOutlined } from '@ant-design/icons';
import dayjs from 'dayjs';
import type { Submission } from '@/types/modules/assignments/submissions';
import { UserAvatar } from '../common';

type Props = {
  submission: Submission & {
    status: 'Pending' | 'Graded';
    path: string;
    percentageMark?: number;
  };
  onClick?: (submission: Submission) => void;
};

const getMarkColor = (mark: number): string => {
  if (mark >= 75) return 'green';
  if (mark >= 50) return 'orange';
  return 'red';
};

const SubmissionListItem = ({ submission, onClick }: Props) => {
  const { user, attempt, status, is_late, percentageMark, created_at, is_practice } = submission;

  const handleClick = () => onClick?.(submission);

  return (
    <List.Item
      key={submission.id}
      className="dark:bg-gray-900 hover:bg-gray-50 dark:hover:bg-gray-800 cursor-pointer transition"
      onClick={handleClick}
      data-cy={`entity-${submission.id}`}
    >
      <List.Item.Meta
        avatar={user ? <UserAvatar user={user} /> : <Avatar icon={<FileTextOutlined />} />}
        title={
          <div className="flex justify-between items-center">
            <span className="font-semibold text-black dark:text-white">
              {user?.username ?? 'Unknown User'}
            </span>
            <Tag color="blue">Attempt #{attempt}</Tag>
          </div>
        }
        description={
          <div className="space-y-1 mt-1">
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
    </List.Item>
  );
};

export default SubmissionListItem;
