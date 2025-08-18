import { List, Avatar } from 'antd';
import { NotificationOutlined } from '@ant-design/icons';
import type { Announcement } from '@/types/modules/announcements';
import React from 'react';
import dayjs from 'dayjs';
import PinnedTag from './PinnedTag';
import { mdExcerpt } from '@/utils/markdown';

interface Props {
  announcement: Announcement;
  onClick?: (a: Announcement) => void;
  actions?: React.ReactNode[];
}

const AnnouncementListItem: React.FC<Props> = ({ announcement, onClick }) => {
  const handleRowClick = () => onClick?.(announcement);

  return (
    <List.Item
      key={announcement.id}
      className="dark:bg-gray-900 hover:bg-gray-50 dark:hover:bg-gray-800 cursor-pointer transition"
      onClick={handleRowClick}
      data-cy="entity-list-item"
    >
      <List.Item.Meta
        avatar={<Avatar icon={<NotificationOutlined />} style={{ backgroundColor: '#faad14' }} />}
        title={
          <div className="flex items-center gap-2 min-w-0">
            <span className="text-black dark:text-white font-medium truncate">
              {announcement.title}
            </span>
            {announcement.pinned && <PinnedTag />}
            <span className="ml-auto text-xs text-gray-500">
              {dayjs(announcement.created_at).format('DD MMM YYYY')}
            </span>
          </div>
        }
        description={
          <div className="w-full text-gray-700 dark:text-neutral-300 !mb-0 line-clamp-2">
            {mdExcerpt(announcement.body || 'No content.', 320)}
          </div>
        }
      />
    </List.Item>
  );
};

export default AnnouncementListItem;
