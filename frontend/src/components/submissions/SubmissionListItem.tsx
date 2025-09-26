// src/components/submissions/SubmissionListItem.tsx
import { List, Avatar, Space, Tag } from 'antd';
import { FileTextOutlined } from '@ant-design/icons';
import dayjs from 'dayjs';
import type { Submission } from '@/types/modules/assignments/submissions';
import { UserAvatar } from '../common';
import SubmissionStatusTag from '@/components/submissions/SubmissionStatusTag';
import PercentageTag from '@/components/common/PercentageTag';

import SubmissionPracticeTag from '@/components/submissions/SubmissionPracticeTag';
import SubmissionIgnoredTag from '@/components/submissions/SubmissionIgnoredTag';
import SubmissionAttemptTag from '@/components/submissions/SubmissionAttemptTag';
import SubmissionLateTag from './SubmissionLateTag';

type Props = {
  submission: Submission & {
    path: string;
    percentagePct?: number;
  };
  onClick?: (submission: Submission) => void;
  /** Student view removes user meta and changes the title/right-side layout */
  isStudent?: boolean;
};

const SubmissionListItem = ({ submission, onClick, isStudent = false }: Props) => {
  const { user, attempt, status, is_late, percentagePct, created_at, is_practice, ignored, mark } =
    submission;

  const handleClick = () => onClick?.(submission);

  const showPct =
    typeof percentagePct === 'number' || (mark && typeof mark.total === 'number' && mark.total > 0);

  const pct =
    typeof percentagePct === 'number'
      ? percentagePct
      : mark && mark.total > 0
        ? Math.round((mark.earned / mark.total) * 100)
        : null;

  // Avatar: show only for staff view
  const metaAvatar = !isStudent ? (
    user ? (
      <UserAvatar user={user} />
    ) : (
      <Avatar icon={<FileTextOutlined />} />
    )
  ) : undefined;

  // Title row differs for staff vs student
  const metaTitle = !isStudent ? (
    // STAFF: Username left, Attempt tag right
    <div className="flex justify-between items-center">
      <span className="font-semibold text-black dark:text-white">
        {user?.username ?? 'Unknown User'}
      </span>
      <SubmissionAttemptTag attempt={attempt} />
    </div>
  ) : (
    // STUDENT: "Attempt #N" (plain text) left, right: Status + Percentage (+ Practice/Ignored)
    <div className="flex justify-between items-center">
      <span className="font-semibold text-black dark:text-white">Attempt #{attempt}</span>
      <Space wrap size={8}>
        <SubmissionPracticeTag practice={is_practice} />
        <SubmissionIgnoredTag ignored={ignored} />
        <SubmissionStatusTag status={status} />
        {showPct ? <PercentageTag value={pct ?? 0} scheme="red-green" /> : <Tag>Not marked</Tag>}
      </Space>
    </div>
  );

  return (
    <List.Item
      key={submission.id}
      className="dark:bg-gray-900 hover:bg-gray-50 dark:hover:bg-gray-800 cursor-pointer transition"
      onClick={handleClick}
      data-cy={`entity-${submission.id}`}
    >
      <List.Item.Meta
        avatar={metaAvatar}
        title={metaTitle}
        description={
          <div className="space-y-1 mt-1">
            {!isStudent && (
              // STAFF: chips row (status, pct, late, practice, ignored)
              <Space wrap>
                {/* Only render these when true */}
                <SubmissionPracticeTag practice={is_practice} />
                <SubmissionIgnoredTag ignored={ignored} />
                <SubmissionStatusTag status={status} />
                {showPct ? (
                  <PercentageTag value={pct ?? 0} scheme="red-green" />
                ) : (
                  <Tag>Not marked</Tag>
                )}
                {/* Late: show only when late; omit "On Time" in this row */}
                <SubmissionLateTag late={is_late} showOnTime={false} />
              </Space>
            )}

            {/* Shared submitted-at line */}
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
