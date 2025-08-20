import { Tag } from 'antd';
import type { TicketStatus } from '@/types/modules/assignments/tickets';

interface Props {
  status: TicketStatus;
  capitalize?: boolean; // Capitalize first letter (default true)
  caps?: boolean; // If true, make ALL CAPS
}

const TicketStatusTag = ({ status, capitalize = true, caps = false }: Props) => {
  const colorMap: Record<TicketStatus, string> = {
    open: 'green',
    closed: 'default',
  };

  let label: string = status;

  if (caps) {
    label = status.toUpperCase();
  } else if (capitalize) {
    label = status.charAt(0).toUpperCase() + status.slice(1).toLowerCase();
  } else {
    label = status.toLowerCase();
  }

  return (
    <Tag color={colorMap[status]} className="font-medium">
      {label}
    </Tag>
  );
};

export default TicketStatusTag;
