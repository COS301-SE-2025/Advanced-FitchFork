// components/announcements/AnnouncementCard.tsx
import { Card, Avatar } from 'antd';
import { NotificationOutlined } from '@ant-design/icons';
import type { Announcement } from '@/types/modules/announcements';
import dayjs from 'dayjs';
import PinnedTag from './PinnedTag';
import ReactMarkdown from 'react-markdown';
import rehypeHighlight from 'rehype-highlight';
import type { Components } from 'react-markdown';

const { Meta } = Card;

interface Props {
  announcement: Announcement;
  actions?: React.ReactNode[];
  onClick?: () => void; // NEW
}

const markdownComponents: Components = {
  p: ({ children }) => <>{children}</>, // keep inline so line clamp works
};

const AnnouncementCard = ({ announcement, actions, onClick }: Props) => {
  return (
    <Card
      hoverable
      onClick={onClick} // NEW
      role="button"
      tabIndex={0}
      className="w-full cursor-pointer !bg-white dark:!bg-gray-900"
      actions={actions}
      data-testid="entity-card"
    >
      <Meta
        avatar={<Avatar icon={<NotificationOutlined />} style={{ backgroundColor: '#faad14' }} />}
        title={
          <div className="flex items-center gap-2 min-w-0">
            <span className="text-black dark:text-white font-medium truncate">
              {announcement.title}
            </span>
            {announcement.pinned && <PinnedTag />}
          </div>
        }
        description={
          <div className="flex flex-col gap-1">
            <div className="prose prose-sm dark:prose-invert max-w-none text-gray-700 dark:text-neutral-300 line-clamp-2">
              <ReactMarkdown rehypePlugins={[rehypeHighlight]} components={markdownComponents}>
                {(announcement.body || 'No content.').replace(/\n+/g, ' ')}
              </ReactMarkdown>
            </div>
            <span className="text-xs text-gray-500">
              Posted {dayjs(announcement.created_at).format('DD MMM YYYY')}
            </span>
          </div>
        }
      />
    </Card>
  );
};

export default AnnouncementCard;
