import { useEffect, useMemo, useState, useCallback } from 'react';
import {
  List,
  Tag,
  Button,
  Space,
  Typography,
  Empty,
  Tooltip,
  Alert,
  Spin,
  message,
  Popconfirm,
} from 'antd';
import { CheckCircleOutlined, MessageOutlined } from '@ant-design/icons';
import { useNavigate } from 'react-router-dom';
import dayjs from 'dayjs';
import relativeTime from 'dayjs/plugin/relativeTime';
import type { SortOption } from '@/types/common';
import type { Ticket, TicketStatus } from '@/types/modules/assignments/tickets';
import type { ModuleRole } from '@/types/modules';
import { getMyTickets } from '@/services/me/tickets/get';
import { closeTicket } from '@/services/modules/assignments/tickets/put';

dayjs.extend(relativeTime);

const { Text, Title } = Typography;

type MyTicketItem = Ticket & {
  user: { id: number; username: string };
  module: { id: number; code: string };
  assignment: { id: number; name: string };
};

const DEFAULT_SORT: SortOption[] = [{ field: 'created_at', order: 'descend' }];

const TicketsPanel: React.FC<{
  role?: ModuleRole;
  year?: number;
  perPage?: number;
  status?: TicketStatus; // default to 'open'
  moduleId?: number;
}> = ({ role, year, perPage = 20, status = 'open', moduleId }) => {
  const navigate = useNavigate();
  const [items, setItems] = useState<MyTicketItem[]>([]);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [pending, setPending] = useState<Set<number>>(new Set()); // per-ticket action state

  // pagination scaffolding (UI pager not shown yet)
  const [page, setPage] = useState(1);
  const [, setTotal] = useState(0);

  const fetchData = useCallback(async () => {
    setLoading(true);
    setError(null);
    try {
      const res = await getMyTickets({
        page,
        per_page: perPage,
        sort: DEFAULT_SORT,
        role,
        year,
        status,
        module_id: moduleId,
      });

      if (!res.success) throw new Error(res.message || 'Failed to load tickets');

      const arr = Array.isArray(res.data.tickets) ? res.data.tickets : [res.data.tickets];

      setItems(arr as MyTicketItem[]);
      setTotal(res.data.total ?? arr.length);
    } catch (e: any) {
      setError(e?.message ?? 'Unknown error');
      setItems([]);
      setTotal(0);
    } finally {
      setLoading(false);
    }
  }, [page, perPage, role, year, status, moduleId]);

  useEffect(() => setPage(1), [status, role, year, perPage, moduleId]);
  useEffect(() => {
    fetchData();
  }, [fetchData]);

  const openTickets = useMemo(() => items.filter((t) => t.status === 'open'), [items]);
  const openCount = openTickets.length;

  const buildTicketPath = (t: MyTicketItem) =>
    `/modules/${t.module.id}/assignments/${t.assignment.id}/tickets/${t.id}`;

  const handleView = (t: MyTicketItem, evt?: React.MouseEvent | React.KeyboardEvent) => {
    const isMeta = (evt as any)?.metaKey || (evt as any)?.ctrlKey;
    const path = buildTicketPath(t);
    if (isMeta) window.open(path, '_blank', 'noopener,noreferrer');
    else navigate(path);
  };

  const handleClose = async (t: MyTicketItem) => {
    if (t.status !== 'open' || pending.has(t.id)) return;

    // optimistic update
    setPending((p) => new Set(p).add(t.id));
    const prev = items;

    setItems((prevItems) =>
      prevItems.map((x) => (x.id === t.id ? { ...x, status: 'closed' as TicketStatus } : x)),
    );

    try {
      const res = await closeTicket(t.module.id, t.assignment.id, t.id);
      if (!res.success || res.data.status !== 'closed') {
        throw new Error(res.message || 'Close failed');
      }
      message.success('Ticket closed');
    } catch (e: any) {
      // rollback
      setItems(prev);
      message.error(e?.message ?? 'Failed to close ticket');
    } finally {
      setPending((p) => {
        const next = new Set(p);
        next.delete(t.id);
        return next;
      });
    }
  };

  return (
    <div className="h-full min-h-0 flex flex-col w-full bg-white dark:bg-gray-900 rounded-md border border-gray-200 dark:border-gray-800">
      {/* Header */}
      <div className="px-3 py-2 border-b border-gray-200 dark:border-gray-800">
        <div className="flex items-center justify-between gap-2">
          <div className="flex items-center gap-2">
            <MessageOutlined className="text-gray-500" />
            <Title level={5} className="!mb-0">
              Tickets
            </Title>
          </div>
          <Tag color="orange">{openCount} open</Tag>
        </div>
      </div>

      {/* Error */}
      {error && (
        <div className="px-3 py-2">
          <Alert type="error" showIcon message="Failed to load tickets" description={error} />
        </div>
      )}

      {/* List */}
      <List
        className="flex-1 overflow-y-auto"
        size="small"
        loading={loading}
        locale={{
          emptyText: <Empty image={Empty.PRESENTED_IMAGE_SIMPLE} description="No open tickets." />,
        }}
        dataSource={openTickets}
        renderItem={(t) => {
          const createdText = t.created_at ? dayjs(t.created_at).fromNow() : '—';
          const isClosing = pending.has(t.id);

          const metaUser = t.user?.username ?? `User ${t.user_id}`;
          const metaAssignment = t.assignment?.name ?? `A-${t.assignment_id}`;
          const metaModule = t.module?.code ?? `M-${t.module?.id ?? ''}`;

          return (
            <List.Item
              className="!px-3 !py-2 cursor-pointer"
              role="button"
              tabIndex={0}
              onClick={(e) => handleView(t, e)}
              onKeyDown={(e) => {
                if (e.key === 'Enter' || e.key === ' ') {
                  e.preventDefault();
                  handleView(t, e);
                }
              }}
            >
              {/* --- Mobile compact (no date) --- */}
              <div className="sm:hidden w-full">
                <div className="flex items-start justify-between gap-2 min-w-0">
                  <div className="min-w-0">
                    <div className="text-sm font-medium truncate">{t.title}</div>
                    <div className="text-xs text-gray-500 dark:text-gray-400 truncate">
                      {metaUser}
                      <span className="mx-1">•</span>
                      <span className="truncate inline-block max-w-[55%] align-bottom">
                        {metaAssignment}
                      </span>
                      <span className="mx-1">•</span>
                      {metaModule}
                    </div>
                  </div>

                  <Space size="small" className="shrink-0">
                    <Popconfirm
                      title="Close this ticket?"
                      description="This will mark the ticket as closed."
                      okText="Close"
                      cancelText="Cancel"
                      okButtonProps={{ danger: true, loading: isClosing }}
                      placement="left"
                      onConfirm={(e) => {
                        e?.stopPropagation();
                        void handleClose(t);
                      }}
                      onCancel={(e) => e?.stopPropagation()}
                      onOpenChange={(_open, e) => {
                        // prevent row navigation when toggling the popconfirm
                        (e as any)?.stopPropagation?.();
                      }}
                    >
                      <Button
                        size="small"
                        type="primary"
                        ghost
                        danger
                        icon={<CheckCircleOutlined />}
                        loading={isClosing}
                        disabled={isClosing}
                        onClick={(e) => e.stopPropagation()} // prevent row click
                      >
                        Close
                      </Button>
                    </Popconfirm>
                  </Space>
                </div>
              </div>

              {/* --- Desktop (shows date) --- */}
              <div className="hidden sm:block w-full">
                <div className="flex sm:items-center sm:justify-between gap-2 min-w-0">
                  <div className="min-w-0 flex flex-col gap-1.5">
                    <Text strong className="truncate">
                      {t.title}
                    </Text>
                    <Text type="secondary" className="text-xs truncate block !text-[12px]">
                      {metaUser} • {metaAssignment} • {metaModule} • {createdText}
                    </Text>
                  </div>

                  <Space size="small" className="shrink-0">
                    <Tooltip title="Close ticket">
                      <Button
                        size="small"
                        type="primary"
                        ghost
                        danger
                        icon={<CheckCircleOutlined />}
                        loading={isClosing}
                        disabled={isClosing}
                        onClick={(e) => {
                          e.stopPropagation();
                          void handleClose(t);
                        }}
                      >
                        Close
                      </Button>
                    </Tooltip>
                  </Space>
                </div>
              </div>
            </List.Item>
          );
        }}
      />

      {loading && openTickets.length > 0 && (
        <div className="py-2 flex justify-center">
          <Spin />
        </div>
      )}
    </div>
  );
};

export default TicketsPanel;
