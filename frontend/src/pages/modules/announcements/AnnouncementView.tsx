import { useEffect, useMemo, useRef, useState } from 'react';
import { useNavigate, useParams } from 'react-router-dom';
import type { Announcement } from '@/types/modules/announcements';
import {
  getAnnouncement,
  updateAnnouncement,
  deleteAnnouncement,
} from '@/services/modules/announcements';
import { Button, Card, Skeleton, Popconfirm, Typography } from 'antd';
import { EditOutlined, DeleteOutlined, NotificationOutlined } from '@ant-design/icons';
import PageHeader from '@/components/PageHeader';
import PinnedTag from '@/components/announcements/PinnedTag';
import { useAuth } from '@/context/AuthContext';
import { useModule } from '@/context/ModuleContext';
import { useBreadcrumbContext } from '@/context/BreadcrumbContext';
import dayjs from 'dayjs';
import { message } from '@/utils/message';
import ReactMarkdown from 'react-markdown';
import rehypeHighlight from 'rehype-highlight';
import { stripMarkdown } from '@/utils/markdown';
import { useViewSlot } from '@/context/ViewSlotContext';

type ShowPayload = {
  announcement: Announcement;
  user: { id: number; username: string };
};

const clamp = (s: string, n = 60) => (s.length <= n ? s : s.slice(0, n - 1) + '…');

const AnnouncementView = () => {
  const { setValue } = useViewSlot();
  const { announcement_id } = useParams<{ announcement_id: string }>();
  const navigate = useNavigate();
  const mod = useModule();
  const auth = useAuth();
  const { setBreadcrumbLabel } = useBreadcrumbContext();

  const moduleId = mod.id;
  const [loading, setLoading] = useState(true);
  const [data, setData] = useState<ShowPayload | null>(null);

  const canManage = auth.isAdmin || auth.isLecturer(moduleId) || auth.isAssistantLecturer(moduleId);

  const load = async () => {
    if (!announcement_id) return;
    setLoading(true);
    const res = await getAnnouncement(moduleId, Number(announcement_id));
    if (res.success) {
      setData(res.data);
      setValue(
        <Typography.Text className="text-base font-medium text-gray-900 dark:text-gray-100 truncate">
          {res.data.announcement.title}
        </Typography.Text>,
      );
    } else {
      message.error(res.message || 'Failed to load announcement');
    }
    setLoading(false);
  };

  useEffect(() => {
    load();
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [announcement_id, moduleId]);

  // breadcrumb (avoid loops)
  const breadcrumbKey = useMemo(
    () => `modules/${moduleId}/announcements/${announcement_id ?? ''}`,
    [moduleId, announcement_id],
  );
  const breadcrumbLabel = useMemo(() => {
    const raw = data?.announcement?.title ?? 'Announcement';
    return clamp(stripMarkdown(raw), 60);
  }, [data]);
  const lastSetRef = useRef<{ key: string; label: string } | null>(null);
  useEffect(() => {
    if (!announcement_id) return;
    const prev = lastSetRef.current;
    if (prev && prev.key === breadcrumbKey && prev.label === breadcrumbLabel) return;
    setBreadcrumbLabel(breadcrumbKey, breadcrumbLabel);
    lastSetRef.current = { key: breadcrumbKey, label: breadcrumbLabel };
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [breadcrumbKey, breadcrumbLabel, announcement_id]);

  const meta = useMemo(() => {
    if (!data) return '';
    const when = dayjs(data.announcement.created_at).format('DD MMM YYYY HH:mm');
    return `by ${data.user.username} • ${when}`;
  }, [data]);

  const onDelete = async () => {
    if (!data) return;
    const res = await deleteAnnouncement(moduleId, data.announcement.id);
    if (res.success) {
      message.success(res.message || 'Announcement deleted');
      navigate(`/modules/${moduleId}/announcements`);
    } else {
      message.error(res.message || 'Delete failed');
    }
  };

  const togglePin = async () => {
    if (!data) return;
    const res = await updateAnnouncement(moduleId, data.announcement.id, {
      title: data.announcement.title,
      body: data.announcement.body,
      pinned: !data.announcement.pinned,
    });
    if (res.success) {
      message.success('Pinned state updated');
      load();
    } else {
      message.error(res.message || 'Update failed');
    }
  };

  return (
    <div className="h-full flex flex-col overflow-hidden">
      <div className="flex-1 overflow-y-auto p-4">
        <div className="w-full space-y-4">
          <PageHeader
            title={data ? clamp(stripMarkdown(data.announcement.title ?? ''), 80) : 'Announcement'}
          />

          <Card className="!bg-white dark:!bg-gray-900 !border-gray-200 dark:!border-gray-800">
            {loading || !data ? (
              <Skeleton active paragraph={{ rows: 6 }} />
            ) : (
              <div className="flex flex-col gap-4">
                {/* header row (responsive) */}
                <div className="flex flex-col sm:flex-row sm:items-center gap-2 sm:gap-3">
                  <div className="flex items-center gap-2 min-w-0">
                    <div className="inline-flex items-center justify-center w-8 h-8 rounded-full bg-amber-500 text-white shrink-0">
                      <NotificationOutlined />
                    </div>
                    <div className="text-sm text-gray-600 dark:text-gray-400 truncate">{meta}</div>

                    {/* desktop actions */}
                    <div className="ml-auto hidden sm:flex items-center gap-2">
                      {data.announcement.pinned && <PinnedTag />}
                      {canManage && (
                        <>
                          <Button size="small" onClick={togglePin} icon={<EditOutlined />}>
                            {data.announcement.pinned ? 'Unpin' : 'Pin'}
                          </Button>
                          <Popconfirm
                            title="Delete this announcement?"
                            okText="Delete"
                            okButtonProps={{ danger: true }}
                            onConfirm={onDelete}
                          >
                            <Button size="small" icon={<DeleteOutlined />} danger>
                              Delete
                            </Button>
                          </Popconfirm>
                        </>
                      )}
                    </div>
                  </div>

                  {/* mobile actions */}
                  <div className="flex sm:hidden items-center gap-2 pt-2 border-t border-gray-200 dark:border-gray-800">
                    {data.announcement.pinned && <PinnedTag />}
                    {canManage && (
                      <div className="ml-auto grid grid-cols-2 gap-2 w-full">
                        <Button
                          size="small"
                          onClick={togglePin}
                          className="w-full"
                          icon={<EditOutlined />}
                        >
                          {data.announcement.pinned ? 'Unpin' : 'Pin'}
                        </Button>
                        <Popconfirm
                          title="Delete this announcement?"
                          okText="Delete"
                          okButtonProps={{ danger: true }}
                          onConfirm={onDelete}
                        >
                          <Button size="small" danger className="w-full" icon={<DeleteOutlined />}>
                            Delete
                          </Button>
                        </Popconfirm>
                      </div>
                    )}
                  </div>
                </div>

                {/* content */}
                <div className="prose dark:prose-invert max-w-none prose-pre:whitespace-pre-wrap prose-pre:break-words break-words">
                  <ReactMarkdown rehypePlugins={[rehypeHighlight]}>
                    {data.announcement.body || '_No content._'}
                  </ReactMarkdown>
                </div>

                <div className="text-xs text-gray-500">
                  Updated {dayjs(data.announcement.updated_at).format('DD MMM YYYY HH:mm')}
                </div>
              </div>
            )}
          </Card>
        </div>
      </div>
    </div>
  );
};

export default AnnouncementView;
