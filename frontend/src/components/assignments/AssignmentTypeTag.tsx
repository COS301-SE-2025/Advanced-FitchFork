import { Tag } from 'antd';
import type { AssignmentType } from '@/types/modules/assignments';

interface Props {
  type: AssignmentType;
}

const typeMeta: Record<AssignmentType, { color: string; label: string }> = {
  assignment: { color: 'green', label: 'Assignment' },
  practical: { color: 'blue', label: 'Practical' },
};

const AssignmentTypeTag = ({ type }: Props) => {
  const { color, label } = typeMeta[type];
  return (
    <Tag color={color} className="text-xs">
      {label}
    </Tag>
  );
};

export default AssignmentTypeTag;
