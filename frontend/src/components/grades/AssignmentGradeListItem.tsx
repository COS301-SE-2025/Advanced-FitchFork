import { List } from 'antd';
import dayjs from 'dayjs';
import type { GradeResponse } from '@/services/modules/assignments/grades';
import { PercentageTag } from '../common';

type Props = {
  grade: GradeResponse;
  onClick?: (grade: GradeResponse) => void;
};

const AssignmentGradeListItem = ({ grade, onClick }: Props) => {
  const handleClick = () => onClick?.(grade);

  return (
    <List.Item
      key={grade.id}
      className="dark:bg-gray-900 hover:bg-gray-50 dark:hover:bg-gray-800 cursor-pointer transition"
      onClick={handleClick}
      data-cy={`grade-${grade.id}`}
    >
      <List.Item.Meta
        title={
          <div className="flex justify-between items-center">
            <span className="font-semibold text-black dark:text-white">
              {grade.username ?? 'Unknown User'}
            </span>
            <PercentageTag value={grade.score} decimals={1} palette="redGreen" />
          </div>
        }
        description={
          <div className="text-xs text-gray-500 dark:text-neutral-400 mt-1">
            Updated: {dayjs(grade.updated_at).format('YYYY-MM-DD HH:mm')}
          </div>
        }
      />
    </List.Item>
  );
};

export default AssignmentGradeListItem;
