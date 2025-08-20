import React from 'react';
import { List, Tag, Avatar } from 'antd';
import { WarningOutlined } from '@ant-design/icons';
import type { PlagiarismCaseItem } from '@/types/modules/assignments/plagiarism';
import dayjs from 'dayjs';

interface Props {
  caseItem: PlagiarismCaseItem;
  onClick?: (c: PlagiarismCaseItem) => void;
  actions?: React.ReactNode[];
}

const statusColors: Record<PlagiarismCaseItem['status'], string> = {
  review: 'blue',
  flagged: 'red',
  reviewed: 'green',
};

const PlagiarismCaseListItem: React.FC<Props> = ({ caseItem, onClick }) => {
  const handleRowClick = () => onClick?.(caseItem);

  return (
    <List.Item
      key={caseItem.id}
      className="dark:bg-gray-900 hover:bg-gray-50 dark:hover:bg-gray-800 cursor-pointer transition"
      onClick={handleRowClick}
      data-cy="plagiarism-list-item"
    >
      <List.Item.Meta
        avatar={
          <Avatar
            icon={<WarningOutlined />}
            style={{ backgroundColor: '#ffd666', color: '#ad2102' }}
          />
        }
        title={
          <div className="flex items-center gap-2 min-w-0">
            <span className="text-black dark:text-white font-medium truncate">
              Case #{caseItem.id} • {caseItem.submission_1.user.username} vs{' '}
              {caseItem.submission_2.user.username}
            </span>
            <Tag color={statusColors[caseItem.status]} className="shrink-0">
              {caseItem.status}
            </Tag>
            <span className="ml-auto text-xs text-gray-500 shrink-0">
              {dayjs(caseItem.updated_at).format('DD MMM YYYY')}
            </span>
          </div>
        }
        description={
          <div className="w-full text-gray-700 dark:text-neutral-300 !mb-0 line-clamp-2">
            {caseItem.description || 'No description.'}
          </div>
        }
      />
    </List.Item>
  );
};

export default PlagiarismCaseListItem;
