import { List, Avatar, Tooltip, Typography, Space } from 'antd';
import { FileTextOutlined, CalendarOutlined, ClockCircleOutlined } from '@ant-design/icons';
import dayjs from 'dayjs';
import relativeTime from 'dayjs/plugin/relativeTime';
import type { Assignment } from '@/types/modules/assignments';
import AssignmentStatusTag from './AssignmentStatusTag';
import AssignmentTypeTag from './AssignmentTypeTag';

dayjs.extend(relativeTime);

const { Text } = Typography;

const ABS_LONG = 'MMM D, YYYY · h:mm A';
const ABS_SHORT = 'D MMM HH:mm';
const fmtAbsShort = (d: dayjs.Dayjs) =>
  d.year() === dayjs().year() ? d.format(ABS_SHORT) : d.format(`D MMM 'YY HH:mm`);

interface Props {
  assignment: Assignment;
  onClick?: (assignment: Assignment) => void;
}

const AssignmentListItem = ({ assignment, onClick }: Props) => {
  const handleClick = () => onClick?.(assignment);

  const openAt = assignment.available_from ? dayjs(assignment.available_from) : null;
  const dueAt = assignment.due_date ? dayjs(assignment.due_date) : null;

  return (
    <List.Item
      key={assignment.id}
      className="dark:bg-gray-900 hover:bg-gray-50 dark:hover:bg-gray-800 cursor-pointer transition"
      onClick={handleClick}
      data-cy={`entity-${assignment.id}`}
    >
      <List.Item.Meta
        avatar={
          <Avatar
            icon={<FileTextOutlined />}
            style={{ backgroundColor: '#3b82f6' }}
            shape="square"
          />
        }
        title={
          <div className="flex justify-between items-center gap-2">
            <Tooltip title={assignment.name}>
              <span className="truncate text-black dark:text-white font-medium">
                {assignment.name}
              </span>
            </Tooltip>
            <div className="flex flex-nowrap gap-1">
              <AssignmentTypeTag type={assignment.assignment_type} />
              <AssignmentStatusTag status={assignment.status} />
            </div>
          </div>
        }
        description={
          <div className="text-gray-700 dark:text-neutral-300">
            <div className="mb-1">
              {assignment.description || (
                <span className="text-gray-400">No description provided.</span>
              )}
            </div>
            <Space
              direction="vertical"
              size={2}
              className="text-xs text-gray-500 dark:text-gray-400"
            >
              {openAt && (
                <div className="flex items-center gap-1">
                  <CalendarOutlined />
                  <Tooltip title={openAt.format(ABS_LONG)}>
                    <span>
                      <Text strong>Opens</Text> {openAt.fromNow(true)}
                    </span>
                  </Tooltip>
                  <span className="mx-1 text-gray-400">·</span>
                  <span>{fmtAbsShort(openAt)}</span>
                </div>
              )}
              {dueAt && (
                <div className="flex items-center gap-1">
                  <ClockCircleOutlined />
                  <Tooltip title={dueAt.format(ABS_LONG)}>
                    <span>
                      <Text strong>{dayjs().isBefore(dueAt) ? 'Due' : 'Overdue'}</Text>{' '}
                      {dayjs().to(dueAt, true)}
                    </span>
                  </Tooltip>
                  <span className="mx-1 text-gray-400">·</span>
                  <span>{fmtAbsShort(dueAt)}</span>
                </div>
              )}
              {!openAt && !dueAt && <span className="text-gray-400">No dates</span>}
            </Space>
          </div>
        }
      />
    </List.Item>
  );
};

export default AssignmentListItem;
