import { Tag } from 'antd';
import type { PlagiarismCaseStatus } from '@/types/modules/assignments/plagiarism';

interface Props {
  status: PlagiarismCaseStatus;
}

const statusColors: Record<PlagiarismCaseStatus, string> = {
  review: 'blue',
  flagged: 'red',
  reviewed: 'green',
};

const PlagiarismStatusTag = ({ status }: Props) => {
  return (
    <Tag color={statusColors[status]} className="font-medium capitalize">
      {status}
    </Tag>
  );
};

export default PlagiarismStatusTag;
