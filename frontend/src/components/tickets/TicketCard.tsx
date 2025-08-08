import { Card, Avatar, Typography } from 'antd';
import { MessageOutlined } from '@ant-design/icons';
import dayjs from 'dayjs';
import type { Ticket } from '@/types/modules/assignments/tickets';
import TicketStatusTag from './TicketStatusTag';

const { Meta } = Card;
const { Paragraph } = Typography;

interface Props {
  ticket: Ticket;
  actions?: React.ReactNode[];
  onClick?: (ticket: Ticket) => void;
}

const TicketCard = ({ ticket, actions, onClick }: Props) => {
  const handleClick = () => {
    onClick?.(ticket);
  };

  return (
    <Card
      hoverable
      onClick={handleClick}
      className="w-full cursor-pointer dark:bg-neutral-800 dark:border-neutral-700"
      actions={actions}
      data-cy={`entity-${ticket.id}`}
    >
      <Meta
        avatar={<Avatar icon={<MessageOutlined />} style={{ backgroundColor: '#1890ff' }} />}
        title={
          <div className="flex justify-between items-center">
            <span className="text-black dark:text-white">{ticket.title}</span>
            <TicketStatusTag status={ticket.status} />
          </div>
        }
        description={
          <div className="text-gray-700 dark:text-neutral-300">
            <Paragraph ellipsis={{ rows: 2 }} className="mb-1">
              {ticket.description || 'No description available.'}
            </Paragraph>
            <div className="text-xs text-gray-500 dark:text-gray-400">
              <div>Created: {dayjs(ticket.created_at).format('YYYY-MM-DD HH:mm')}</div>
              <div>Updated: {dayjs(ticket.updated_at).format('YYYY-MM-DD HH:mm')}</div>
            </div>
          </div>
        }
      />
    </Card>
  );
};

export default TicketCard;
