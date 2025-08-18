import { Card } from 'antd';
import { WarningOutlined } from '@ant-design/icons';
import type { PlagiarismCaseItem } from '@/types/modules/assignments/plagiarism';
import dayjs from 'dayjs';
import PlagiarismStatusTag from './PlagiarismStatusTag';

const { Meta } = Card;

interface Props {
  caseItem: PlagiarismCaseItem;
  actions?: React.ReactNode[];
  onClick?: () => void;
}

const statusColors: Record<string, string> = {
  review: 'blue',
  flagged: 'red',
  reviewed: 'green',
};

const PlagiarismCaseCard = ({ caseItem, actions, onClick }: Props) => {
  return (
    <Card
      hoverable
      onClick={onClick}
      role="button"
      tabIndex={0}
      className="w-full cursor-pointer !bg-white dark:!bg-gray-900"
      actions={actions}
      data-testid="plagiarism-case-card"
    >
      <Meta
        avatar={<WarningOutlined style={{ fontSize: 24, color: statusColors[caseItem.status] }} />}
        title={
          <div className="flex items-center gap-2 min-w-0">
            <span className="text-black dark:text-white font-medium truncate">
              Case #{caseItem.id}
            </span>
            <PlagiarismStatusTag status={caseItem.status} />
          </div>
        }
        description={
          <div className="flex flex-col gap-1">
            <p className="line-clamp-2 text-gray-700 dark:text-neutral-300">
              {caseItem.description || 'No description'}
            </p>
            <span className="text-xs text-gray-500">
              Submissions: {caseItem.submission_1.user.username} vs{' '}
              {caseItem.submission_2.user.username}
            </span>
            <span className="text-xs text-gray-500">
              Updated {dayjs(caseItem.updated_at).format('DD MMM YYYY HH:mm')}
            </span>
          </div>
        }
      />
    </Card>
  );
};

export default PlagiarismCaseCard;
