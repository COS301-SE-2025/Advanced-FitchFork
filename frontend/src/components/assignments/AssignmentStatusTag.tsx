import { Tag } from 'antd';
import type { AssignmentStatus } from '@/types/modules/assignments';

interface Props {
  status: AssignmentStatus;
}

const statusMeta: Record<AssignmentStatus, { color: string; label: string }> = {
  setup: { color: 'default', label: 'Setup' },
  ready: { color: 'processing', label: 'Ready' },
  open: { color: 'green', label: 'Open' },
  closed: { color: 'volcano', label: 'Closed' },
  archived: { color: 'gray', label: 'Archived' },
};

const AssignmentStatusTag = ({ status }: Props) => {
  const { color, label } = statusMeta[status];

  return (
    <Tag color={color} className="text-xs">
      {label}
    </Tag>
  );
};

export default AssignmentStatusTag;
