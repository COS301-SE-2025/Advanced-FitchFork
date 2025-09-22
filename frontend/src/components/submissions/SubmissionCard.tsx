// src/components/submissions/SubmissionCard.tsx
import { Card, Tag, Space, Avatar } from 'antd';
import { FileTextOutlined } from '@ant-design/icons';
import dayjs from 'dayjs';
import type { ReactNode } from 'react';
import type { Submission } from '@/types/modules/assignments/submissions';
import { UserAvatar } from '@/components/common';
import SubmissionStatusTag from '@/components/submissions/SubmissionStatusTag';
import PercentageTag from '@/components/common/PercentageTag';

type Props = {
  submission: Submission & {
    path: string;
    percentagePct?: number;
  };
  actions?: ReactNode[];
  onClick?: (submission: Submission) => void;
};

const SubmissionCard = ({ submission, actions = [], onClick }: Props) => {
  const { user, attempt, status, is_late, percentagePct, created_at, is_practice, mark } =
    submission;

  const showPct =
    typeof percentagePct === 'number' || (mark && typeof mark.total === 'number' && mark.total > 0);

  const pct =
    typeof percentagePct === 'number'
      ? percentagePct
      : mark && mark.total > 0
        ? Math.round((mark.earned / mark.total) * 100)
        : null;

  return (
    <Card
      hoverable
      actions={actions}
      className="rounded-lg border border-gray-200 dark:border-gray-700"
      onClick={() => onClick?.(submission)}
    >
      <Card.Meta
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
          <div className="space-y-2 mt-2">
            <Space wrap>
              <SubmissionStatusTag status={status} />
              {showPct ? (
                <PercentageTag value={pct ?? 0} palette="redGreen" />
              ) : (
                <Tag>Not marked</Tag>
              )}
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
