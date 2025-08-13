import { useMemo, useState } from 'react';
import { List, Typography, Segmented, Empty, Tooltip } from 'antd';
import { NotificationOutlined, PushpinFilled, ClockCircleOutlined } from '@ant-design/icons';
import dayjs from 'dayjs';
import relativeTime from 'dayjs/plugin/relativeTime';
import type { Announcement } from '@/types/modules/announcements/shared';
import type { User } from '@/types/users';
import type { Module } from '@/types/modules';
import PinnedTag from './PinnedTag';

dayjs.extend(relativeTime);

const { Text, Title } = Typography;

const MOCK_USERS: Record<number, User> = {
  1: { id: 1, username: 'dr_smith', email: '', admin: false, created_at: '', updated_at: '' },
  2: { id: 2, username: 'tutor_jane', email: '', admin: false, created_at: '', updated_at: '' },
};

const MOCK_MODULES: Record<number, Module> = {
  101: {
    id: 101,
    code: 'COS101',
    year: 2025,
    description: '',
    credits: 16,
    created_at: '',
    updated_at: '',
  },
  344: {
    id: 344,
    code: 'COS344',
    year: 2025,
    description: '',
    credits: 24,
    created_at: '',
    updated_at: '',
  },
};

const nowISO = (offsetMs = 0) => new Date(Date.now() - offsetMs).toISOString();

const MOCK_ANNOUNCEMENTS: Announcement[] = [
  {
    id: 9003,
    module_id: 344,
    user_id: 1,
    title: 'Assignment 2 Released',
    body: 'A2 is now available.',
    pinned: true,
    created_at: nowISO(60 * 60 * 1000),
    updated_at: nowISO(60 * 60 * 1000),
  },
  {
    id: 9002,
    module_id: 101,
    user_id: 2,
    title: 'Tutorial Venue Change',
    body: 'This week’s tutorial moves to Lab 3.',
    pinned: false,
    created_at: nowISO(26 * 60 * 60 * 1000),
    updated_at: nowISO(20 * 60 * 60 * 1000),
  },
  {
    id: 9004,
    module_id: 101,
    user_id: 1,
    title: 'Exam Preparation Material Uploaded',
    body: 'Find the revision pack under Resources.',
    pinned: true,
    created_at: nowISO(3 * 24 * 60 * 60 * 1000),
    updated_at: nowISO(3 * 24 * 60 * 60 * 1000),
  },
  {
    id: 9005,
    module_id: 344,
    user_id: 2,
    title: 'Lab Schedule Updated',
    body: 'Lab times have shifted to accommodate public holidays.',
    pinned: false,
    created_at: nowISO(5 * 24 * 60 * 60 * 1000),
    updated_at: nowISO(4.5 * 24 * 60 * 60 * 1000),
  },
  {
    id: 9006,
    module_id: 101,
    user_id: 2,
    title: 'Group Project Guidelines',
    body: 'Please review the updated group work rules.',
    pinned: false,
    created_at: nowISO(7 * 24 * 60 * 60 * 1000),
    updated_at: nowISO(7 * 24 * 60 * 60 * 1000),
  },
  {
    id: 9007,
    module_id: 344,
    user_id: 1,
    title: 'Midterm Grades Published',
    body: 'Check the Grades tab for your results.',
    pinned: true,
    created_at: nowISO(10 * 24 * 60 * 60 * 1000),
    updated_at: nowISO(10 * 24 * 60 * 60 * 1000),
  },
  {
    id: 9008,
    module_id: 101,
    user_id: 1,
    title: 'Weekly Quiz Reminder',
    body: 'Quiz closes this Friday at 5pm.',
    pinned: false,
    created_at: nowISO(12 * 24 * 60 * 60 * 1000),
    updated_at: nowISO(12 * 24 * 60 * 60 * 1000),
  },
];

const AnnouncementsPanel: React.FC = () => {
  const [announcements] = useState<Announcement[]>(MOCK_ANNOUNCEMENTS);
  const [filter, setFilter] = useState<'all' | 'pinned'>('all');

  const filtered = useMemo(() => {
    const sorted = [...announcements].sort((a, b) => {
      if (a.pinned !== b.pinned) return a.pinned ? -1 : 1;
      return dayjs(b.created_at).valueOf() - dayjs(a.created_at).valueOf();
    });
    return sorted.filter((a) => (filter === 'all' ? true : a.pinned));
  }, [announcements, filter]);

  const filterOptions = [
    { label: 'All', value: 'all' },
    { label: 'Pinned', value: 'pinned', icon: <PushpinFilled /> },
  ];

  return (
    <div className="h-full min-h-0 flex flex-col w-full bg-white dark:bg-gray-900 rounded-md border border-gray-200 dark:border-gray-800">
      {/* Header */}
      <div className="px-3 py-2 border-b border-gray-200 dark:border-gray-800">
        <div className="flex items-center justify-between gap-2">
          <div className="flex items-center gap-2">
            <NotificationOutlined className="text-gray-500" />
            <Title level={5} className="!mb-0">
              Announcements
            </Title>
          </div>

          <Segmented
            value={filter}
            onChange={(v) => setFilter(v as 'all' | 'pinned')}
            options={filterOptions}
            size="small"
            className="role-seg"
          />
        </div>
      </div>

      {/* List */}
      <List
        className="flex-1 overflow-y-auto"
        locale={{
          emptyText: (
            <Empty
              image={Empty.PRESENTED_IMAGE_SIMPLE}
              description={filter === 'pinned' ? 'No pinned announcements.' : 'No announcements.'}
            />
          ),
        }}
        dataSource={filtered}
        renderItem={(a) => {
          const author = MOCK_USERS[a.user_id]?.username ?? `User ${a.user_id}`;
          const moduleCode = MOCK_MODULES[a.module_id]?.code ?? `M-${a.module_id}`;
          const created = dayjs(a.created_at);

          return (
            <List.Item
              className="!px-3 cursor-pointer"
              onClick={() => console.log('Open announcement', { id: a.id })}
            >
              <div className="flex flex-col gap-1.5 w-full">
                {/* Title + pin */}
                <div className="flex items-center justify-between gap-2 min-w-0">
                  <Text strong className="truncate">
                    {a.title}
                  </Text>
                  {a.pinned && <PinnedTag />}
                </div>

                {/* Meta */}
                <div className="flex flex-wrap items-center gap-x-2 gap-y-1">
                  <Text type="secondary" className="!text-[12px]">
                    {moduleCode}
                  </Text>
                  <Text type="secondary" className="!text-[12px]">
                    •
                  </Text>
                  <Text type="secondary" className="!text-[12px]">
                    {author}
                  </Text>
                  <Text type="secondary" className="!text-[12px]">
                    •
                  </Text>
                  <Text type="secondary" className="inline-flex items-center !text-[12px]">
                    <Tooltip title={created.format('YYYY-MM-DD HH:mm')}>
                      <ClockCircleOutlined className="mr-1" />
                    </Tooltip>
                    {created.fromNow()}
                  </Text>
                </div>
              </div>
            </List.Item>
          );
        }}
      />
    </div>
  );
};

export default AnnouncementsPanel;
