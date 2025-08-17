import { useParams } from 'react-router-dom';
import { useModule } from '@/context/ModuleContext';
import { useAssignment } from '@/context/AssignmentContext';
import { Typography, Descriptions, Skeleton, Button, Popconfirm } from 'antd';
import { InfoCircleOutlined } from '@ant-design/icons';
import type { Ticket } from '@/types/modules/assignments/tickets';
import type { User } from '@/types/users';
import { useEffect, useState } from 'react';
import { getTicket } from '@/services/modules/assignments/tickets/get';
import TicketStatusTag from '@/components/tickets/TicketStatusTag';
import { closeTicket, openTicket } from '@/services/modules/assignments/tickets/put';
import UserAvatar from '@/components/common/UserAvatar';
import { useBreadcrumbContext } from '@/context/BreadcrumbContext';
import { useViewSlot } from '@/context/ViewSlotContext';
import { TicketChat } from '@/components/tickets';

const { Title, Text, Paragraph } = Typography;

const TicketView = () => {
  const { ticket_id } = useParams();
  const ticketId = Number(ticket_id);
  const module = useModule();
  const { assignment } = useAssignment();
  const { setValue } = useViewSlot();
  const { setBreadcrumbLabel } = useBreadcrumbContext();

  const [ticket, setTicket] = useState<Ticket | null>(null);
  const [user, setUser] = useState<User | null>(null);
  const [loading, setLoading] = useState(true);
  const [updatingStatus, setUpdatingStatus] = useState(false);
  const [showHeaderInfo, setShowHeaderInfo] = useState(false); // NEW toggle state

  const fetchTicket = async () => {
    if (!module || !assignment || !ticket_id) return;
    setLoading(true);
    try {
      const res = await getTicket(module.id, assignment.id, Number(ticket_id));

      if (res.success && res.data) {
        setTicket(res.data.ticket);
        setBreadcrumbLabel(
          `modules/${module.id}/assignments/${assignment.id}/tickets/${ticketId}`,
          res.data.ticket.title,
        );
        setUser(res.data.user);
      }
    } finally {
      setLoading(false);
    }
  };

  useEffect(() => {
    const maxLength = 40;
    const trimmedTitle =
      ticket?.title && ticket.title.length > maxLength
        ? `${ticket.title.slice(0, maxLength)}…`
        : ticket?.title || `Ticket #${ticket_id}`;

    setValue(
      <div className="flex items-center justify-between w-full gap-2">
        {/* Left: title + status tag */}
        <div className="flex items-center gap-2 min-w-0">
          <Typography.Text
            className="text-base font-medium text-gray-900 dark:text-gray-100"
            title={ticket?.title}
          >
            {trimmedTitle}
          </Typography.Text>
          {ticket && <TicketStatusTag status={ticket.status} />}
        </div>

        {/* Right: info button */}
        <Button
          type="link"
          size="small"
          icon={<InfoCircleOutlined />}
          onClick={() => setShowHeaderInfo((v) => !v)}
          className="!block md:!hidden shrink-0"
        />
      </div>,
    );
  }, [ticket]);

  useEffect(() => {
    fetchTicket();
  }, [module, assignment, ticket_id]);

  const handleToggleStatus = async () => {
    if (!ticket || !module || !assignment) return;
    setUpdatingStatus(true);
    try {
      const action = ticket.status === 'open' ? closeTicket : openTicket;
      const res = await action(module.id, assignment.id, ticket.id);
      setTicket((prev) => (prev ? { ...prev, status: res.data.status } : prev));
    } finally {
      setUpdatingStatus(false);
    }
  };

  if (!ticket_id || !assignment || !module) return null;

  const TicketInfoContent = (
    <Skeleton active loading={loading} paragraph={{ rows: 6 }}>
      <div className="space-y-6">
        {user && (
          <div className="flex items-center gap-3">
            <UserAvatar user={user} />
            <div>
              <Text strong className="block">
                {user.username}
              </Text>
              <Text type="secondary" className="text-xs">
                {user.email}
              </Text>
            </div>
          </div>
        )}

        <div>
          <Title level={5} className="!mb-3 text-gray-800 dark:text-gray-100">
            Ticket Info
          </Title>
          <Descriptions
            size="small"
            column={1}
            labelStyle={{ fontWeight: 500 }}
            contentStyle={{ color: 'inherit' }}
          >
            <Descriptions.Item label="Ticket ID">{ticket?.id ?? ticket_id}</Descriptions.Item>
            <Descriptions.Item label="Created">
              {ticket?.created_at ? new Date(ticket.created_at).toLocaleString() : '–'}
            </Descriptions.Item>
          </Descriptions>
        </div>

        {ticket?.description && (
          <div>
            <Title level={5} className="!mb-2 text-gray-800 dark:text-gray-100">
              Description
            </Title>
            <Paragraph className="text-sm text-gray-700 dark:text-gray-300 whitespace-pre-wrap">
              {ticket.description}
            </Paragraph>
          </div>
        )}

        {ticket &&
          (ticket.status === 'open' ? (
            <Popconfirm
              title="Close this ticket?"
              description="Are you sre your issue has been resolved?"
              okText="Yes"
              okType="danger"
              cancelText="No"
              onConfirm={handleToggleStatus}
            >
              <Button block danger loading={updatingStatus}>
                Close Ticket
              </Button>
            </Popconfirm>
          ) : (
            <Button block type="primary" loading={updatingStatus} onClick={handleToggleStatus}>
              Reopen Ticket
            </Button>
          ))}
      </div>
    </Skeleton>
  );

  return (
    <div className="flex-1 flex flex-col min-h-0 overflow-hidden">
      {/* Header (desktop only) */}
      <div className="hidden md:block bg-white dark:bg-gray-900 px-4 py-5 border-b border-gray-200 dark:border-gray-800">
        <div className="flex items-start justify-between w-full flex-wrap gap-2">
          <div className="flex items-center gap-2 flex-wrap">
            <Title
              level={5}
              className="!mb-0 !text-gray-900 dark:!text-gray-100 whitespace-normal break-words"
            >
              {ticket?.title || `Ticket #${ticket_id}`}
            </Title>
            {ticket && <TicketStatusTag status={ticket.status} />}
          </div>

          {/* Mobile-only toggle button */}
          <Button
            type="default"
            size="small"
            icon={<InfoCircleOutlined />}
            onClick={() => setShowHeaderInfo((v) => !v)}
            className="!block md:!hidden"
          />
        </div>
      </div>

      {/* Mobile-only ticket details below header */}
      {showHeaderInfo && (
        <div className="block md:hidden px-4 pb-4 pt-3 border-b border-gray-200 dark:border-gray-800 bg-white dark:bg-gray-950">
          {TicketInfoContent}
        </div>
      )}

      {/* Layout */}
      <div className="flex-1 min-h-0 flex flex-col-reverse md:flex-row overflow-hidden">
        {/* Left (chat area) */}
        <div className="flex-1 min-h-0 flex flex-col border-r border-gray-200 dark:border-gray-800">
          {ticket ? (
            <TicketChat ticket={ticket} />
          ) : (
            // Fallback while loading / not found yet
            <div className="flex-1 p-4">
              <Skeleton active paragraph={{ rows: 10 }} className="w-full " />
            </div>
          )}
        </div>

        {/* Right (desktop sidebar) */}
        <div className="hidden md:block w-full md:w-[380px] shrink-0 p-4 bg-white dark:bg-gray-900 border-t md:border-t-0 md:border-l-0 border-gray-200 dark:border-gray-800 overflow-auto">
          {TicketInfoContent}
        </div>
      </div>
    </div>
  );
};

export default TicketView;
