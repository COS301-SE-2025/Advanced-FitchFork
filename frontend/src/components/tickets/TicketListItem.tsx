import { List, Avatar, Typography } from 'antd';
import { MessageOutlined } from '@ant-design/icons';
import type { Ticket } from '@/types/modules/assignments/tickets';
import TicketStatusTag from '@/components/tickets/TicketStatusTag';

const { Text } = Typography;

interface Props {
  ticket: Ticket;
  onClick?: (ticket: Ticket) => void;
}

const TicketListItem = ({ ticket, onClick }: Props) => {
  const handleClick = () => onClick?.(ticket);

  return (
    <List.Item
      key={ticket.id}
      className="dark:bg-gray-900 hover:bg-gray-50 dark:hover:bg-gray-800 cursor-pointer transition"
      onClick={handleClick}
      data-cy={`entity-${ticket.id}`}
    >
      <List.Item.Meta
        avatar={<Avatar icon={<MessageOutlined />} style={{ backgroundColor: '#1890ff' }} />}
        title={
          <div className="flex items-center justify-between">
            <span className="text-black dark:text-white">{ticket.title}</span>
            <TicketStatusTag status={ticket.status} />
          </div>
        }
        description={
          <div className="text-gray-700 dark:text-neutral-300">
            {ticket.description ? (
              <Text className="!mb-0">{ticket.description}</Text>
            ) : (
              <span className="text-gray-400">No description available.</span>
            )}
          </div>
        }
      />
    </List.Item>
  );
};

export default TicketListItem;
