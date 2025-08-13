import { useMemo, useState } from 'react';
import { List, Tag, Button, Space, Typography, Empty, Tooltip } from 'antd';
import { CheckCircleOutlined, MessageOutlined } from '@ant-design/icons'; // Added icon
import dayjs from 'dayjs';
import type { Ticket } from '@/types/modules/assignments/tickets';
import type { Assignment } from '@/types/modules/assignments';
import type { User } from '@/types/users';

const { Text, Title } = Typography;

type IdMap<T> = Record<number, T>;

const TicketsPanel = () => {
  const MOCK_TICKETS: Ticket[] = [
    {
      id: 101,
      assignment_id: 12,
      user_id: 55,
      title: 'Issue with submission grading',
      description: 'My submission shows an error but runs fine locally.',
      status: 'open',
      created_at: new Date().toISOString(),
      updated_at: new Date().toISOString(),
    } as any,
    {
      id: 103,
      assignment_id: 12,
      user_id: 55,
      title: 'Cannot see rubric',
      description: 'The rubric panel is empty.',
      status: 'open',
      created_at: new Date(Date.now() - 7200_000).toISOString(),
      updated_at: new Date().toISOString(),
    } as any,
    {
      id: 102,
      assignment_id: 14,
      user_id: 55,
      title: 'Clarification on question 3',
      description: '',
      status: 'closed',
      created_at: new Date(Date.now() - 3600_000).toISOString(),
      updated_at: new Date().toISOString(),
    } as any,
  ];

  const MOCK_USERS: User[] = [
    {
      id: 55,
      username: 'alice',
      email: 'alice@u.edu',
      admin: false,
      created_at: '',
      updated_at: '',
    } as any,
  ];

  const MOCK_ASSIGNMENTS: Assignment[] = [
    {
      id: 12,
      module_id: 1,
      name: 'COS101 A1',
      description: '',
      assignment_type: 'assignment' as any,
      available_from: '',
      due_date: '',
      status: 'open' as any,
      created_at: '',
      updated_at: '',
    } as any,
    {
      id: 14,
      module_id: 1,
      name: 'COS101 A2',
      description: '',
      assignment_type: 'assignment' as any,
      available_from: '',
      due_date: '',
      status: 'open' as any,
      created_at: '',
      updated_at: '',
    } as any,
  ];

  const [tickets, setTickets] = useState<Ticket[]>(MOCK_TICKETS);
  const [usersById] = useState<IdMap<User>>(Object.fromEntries(MOCK_USERS.map((u) => [u.id, u])));
  const [assignmentsById] = useState<IdMap<Assignment>>(
    Object.fromEntries(MOCK_ASSIGNMENTS.map((a) => [a.id, a])),
  );

  const openTickets = useMemo(() => tickets.filter((t) => t.status === 'open'), [tickets]);
  const openCount = openTickets.length;

  const handleView = (t: Ticket) => {
    console.log('Open ticket', t);
  };

  const handleClose = (t: Ticket) => {
    if (t.status !== 'open') return;
    setTickets((prev) => prev.map((x) => (x.id === t.id ? { ...x, status: 'closed' } : x)));
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

      {/* List */}
      <List
        className="flex-1 overflow-y-auto"
        locale={{
          emptyText: <Empty image={Empty.PRESENTED_IMAGE_SIMPLE} description="No open tickets." />,
        }}
        dataSource={openTickets}
        renderItem={(t) => {
          const userName = usersById[t.user_id]?.username ?? `User ${t.user_id}`;
          const assignmentName = assignmentsById[t.assignment_id]?.name ?? `A-${t.assignment_id}`;
          const createdText = t.created_at
            ? (dayjs(t.created_at).fromNow?.() ?? dayjs(t.created_at).format('YYYY-MM-DD HH:mm'))
            : '—';

          return (
            <List.Item
              className="!px-3 cursor-pointer"
              onClick={() => handleView(t)}
              onKeyDown={(e) => {
                if (e.key === 'Enter' || e.key === ' ') handleView(t);
              }}
            >
              <div className="flex flex-col sm:flex-row sm:items-center sm:justify-between gap-2 w-full">
                {/* Left: title + meta */}
                <div className="min-w-0 flex flex-col gap-1.5">
                  <Text strong className="truncate">
                    {t.title}
                  </Text>
                  <Text type="secondary" className="text-xs truncate block !text-[12px]">
                    {userName} • {assignmentName} • {createdText}
                  </Text>
                </div>

                {/* Right: actions */}
                <Space size="small" className="shrink-0">
                  <Tooltip title="Close ticket">
                    <Button
                      size="small"
                      type="primary"
                      ghost
                      danger
                      icon={<CheckCircleOutlined />}
                      onClick={(e) => {
                        e.stopPropagation();
                        handleClose(t);
                      }}
                    >
                      Close
                    </Button>
                  </Tooltip>
                </Space>
              </div>
            </List.Item>
          );
        }}
      />
    </div>
  );
};

export default TicketsPanel;
