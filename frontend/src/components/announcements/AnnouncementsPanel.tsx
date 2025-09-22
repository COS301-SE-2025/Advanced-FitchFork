import { useEffect, useMemo, useState, useCallback } from 'react';
import { List, Typography, Segmented, Empty, Tooltip, Spin, Alert } from 'antd';
import { NotificationOutlined, PushpinFilled, ClockCircleOutlined } from '@ant-design/icons';
import dayjs from 'dayjs';
import relativeTime from 'dayjs/plugin/relativeTime';
import type { Announcement } from '@/types/modules/announcements/shared';
import type { ModuleRole } from '@/types/modules';
import type { SortOption } from '@/types/common';
import PinnedTag from './PinnedTag';
import { getMyAnnouncements } from '@/services/me/announcements/get';
import { useNavigate } from 'react-router-dom';

dayjs.extend(relativeTime);

const { Text, Title } = Typography;

/** Shape returned by service for each row (augment with optional module code if backend adds it later) */
type MyAnnouncementItem = Announcement & {
  user: { id: number; username: string };
  module?: { id: number; code?: string }; // optional defensive field
};

const DEFAULT_SORT: SortOption[] = [{ field: 'created_at', order: 'descend' }];

const AnnouncementsPanel: React.FC<{
  /** Optional server-side filters you may want to pass down (role/year) */
  role?: ModuleRole;
  year?: number;
  perPage?: number;
  moduleId?: number;
}> = ({ role, year, perPage = 20, moduleId }) => {
  const navigate = useNavigate();
  const [filter, setFilter] = useState<'all' | 'pinned'>('all');
  const [items, setItems] = useState<MyAnnouncementItem[]>([]);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  // basic pagination state (not shown in UI yet; ready if you add <List pagination> later)
  const [page, setPage] = useState(1);
  const [, setTotal] = useState(0);

  const fetchData = useCallback(async () => {
    setLoading(true);
    setError(null);
    try {
      const res = await getMyAnnouncements({
        page,
        per_page: perPage,
        sort: DEFAULT_SORT,
        role,
        year,
        module_id: moduleId,
        pinned: filter === 'pinned' ? true : undefined, // let server filter pinned when needed
      });

      if (!res.success) {
        throw new Error(res.message || 'Failed to load announcements');
      }

      // Defensive: some backends accidentally return a single object instead of an array
      const arr = Array.isArray(res.data.announcements)
        ? res.data.announcements
        : [res.data.announcements];

      setItems(arr as MyAnnouncementItem[]);
      setTotal(res.data.total ?? arr.length);
    } catch (e: any) {
      setError(e?.message ?? 'Unknown error');
      setItems([]);
      setTotal(0);
    } finally {
      setLoading(false);
    }
  }, [page, perPage, role, year, moduleId, filter]);

  useEffect(() => {
    // reset to page 1 when filter changes
    setPage(1);
  }, [filter]);

  useEffect(() => {
    fetchData();
  }, [fetchData]);

  // Keep pinned-first & recent-first ordering on the client as a UX guarantee
  const ordered = useMemo(() => {
    const sorted = [...items].sort((a, b) => {
      if (a.pinned !== b.pinned) return a.pinned ? -1 : 1;
      return dayjs(b.created_at).valueOf() - dayjs(a.created_at).valueOf();
    });
    return sorted;
  }, [items]);

  const filterOptions = [
    { label: 'All', value: 'all' },
    { label: 'Pinned', value: 'pinned', icon: <PushpinFilled /> },
  ];

  return (
    <div className="h-full min-h-0 flex flex-col overflow-hidden w-full bg-white dark:bg-gray-900 rounded-md border border-gray-200 dark:border-gray-800">
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

      {/* Error state */}
      {error && (
        <div className="px-3 py-2">
          <Alert type="error" showIcon message="Failed to load announcements" description={error} />
        </div>
      )}

      {/* List */}
      <List
        className="flex-1 overflow-y-auto"
        loading={loading}
        locale={{
          emptyText: (
            <Empty
              image={Empty.PRESENTED_IMAGE_SIMPLE}
              description={filter === 'pinned' ? 'No pinned announcements.' : 'No announcements.'}
            />
          ),
        }}
        dataSource={ordered}
        renderItem={(a) => {
          const author = a.user?.username ?? `User ${a.user?.id ?? a.user_id}`;
          const moduleCode = a.module?.code ?? `M-${a.module_id}`;
          const created = dayjs(a.created_at);

          return (
            <List.Item
              className="!px-3 cursor-pointer"
              onClick={() => navigate(`/modules/${a.module?.id}/announcements/${a.id}`)}
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

      {/* Optional: simple footer spinner for future infinite scroll / pagination */}
      {loading && items.length > 0 && (
        <div className="py-2 flex justify-center">
          <Spin />
        </div>
      )}
    </div>
  );
};

export default AnnouncementsPanel;
