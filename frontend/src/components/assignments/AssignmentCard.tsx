import { Card, Space, Typography, Tooltip } from 'antd';
import { CalendarOutlined, ClockCircleOutlined, FileTextOutlined } from '@ant-design/icons';
import dayjs from 'dayjs';
import relativeTime from 'dayjs/plugin/relativeTime';
import type { Assignment } from '@/types/modules/assignments';
import { useModule } from '@/context/ModuleContext';
import { useNavigate } from 'react-router-dom';
import AssignmentStatusTag from './AssignmentStatusTag';
import AssignmentTypeTag from './AssignmentTypeTag';
import { useUI } from '@/context/UIContext';

dayjs.extend(relativeTime);

const { Text, Paragraph } = Typography;

interface Props {
  assignment: Assignment;
  actions?: React.ReactNode[];
}

const ABS_LONG = 'MMM D, YYYY · h:mm A';
const ABS_SHORT = 'D MMM HH:mm';
const fmtAbsShort = (d: dayjs.Dayjs) =>
  d.year() === dayjs().year() ? d.format(ABS_SHORT) : d.format(`D MMM 'YY HH:mm`);

const AssignmentCard = ({ assignment, actions }: Props) => {
  const navigate = useNavigate();
  const module = useModule();
  const { isMobile } = useUI();

  const handleClick = () => {
    navigate(`/modules/${module.id}/assignments/${assignment.id}`);
  };

  const header = (
    <div className="flex items-start gap-2">
      <div className="min-w-0 flex items-start gap-2">
        <FileTextOutlined
          className={`mt-0.5 text-blue-500 flex-shrink-0 ${isMobile ? 'text-base' : 'text-lg'}`}
        />
        <Tooltip title={assignment.name} placement="topLeft">
          <span className={`font-medium truncate block ${isMobile ? 'text-sm' : 'text-base'}`}>
            {assignment.name}
          </span>
        </Tooltip>
      </div>
      <div className="ml-auto flex flex-nowrap items-center gap-1 justify-end">
        <AssignmentTypeTag type={assignment.assignment_type} />
        <AssignmentStatusTag status={assignment.status} />
      </div>
    </div>
  );

  const openAt = assignment.available_from ? dayjs(assignment.available_from) : null;
  const dueAt = assignment.due_date ? dayjs(assignment.due_date) : null;

  return (
    <Card
      size={isMobile ? 'small' : 'default'}
      className="dark:!bg-gray-900 dark:border-none cursor-pointer transition-shadow duration-200 hover:shadow-lg"
      onClick={handleClick}
      title={header}
      actions={
        actions && actions.length ? actions.map((a, i) => <div key={i}>{a}</div>) : undefined
      }
      styles={{
        header: isMobile ? { padding: '8px 12px' } : undefined,
        body: isMobile ? { padding: '12px' } : undefined,
        actions: { margin: 0, padding: 0 },
      }}
    >
      <Paragraph
        className={`text-gray-700 dark:text-gray-300 mb-3 ${
          isMobile ? 'text-[13px] line-clamp-3' : 'text-sm'
        }`}
      >
        {assignment.description || 'No description provided.'}
      </Paragraph>

      <Space
        direction="vertical"
        size={6}
        className={`${isMobile ? 'text-[13px]' : 'text-sm'} w-full`}
      >
        {/* Opens row */}
        {openAt && (
          <div className="flex items-center gap-2 text-gray-500 dark:text-gray-400">
            <CalendarOutlined className="flex-shrink-0" />
            <div className="min-w-0 flex-1 truncate">
              <Tooltip title={openAt.format(ABS_LONG)}>
                <span className="whitespace-nowrap">
                  <Text strong>Opens</Text> {openAt.fromNow(true)}
                </span>
              </Tooltip>
              <span className="mx-1 text-gray-400">·</span>
              <span className="whitespace-nowrap">{fmtAbsShort(openAt)}</span>
            </div>
          </div>
        )}

        {/* Due row */}
        {dueAt && (
          <div className="flex items-center gap-2 text-gray-500 dark:text-gray-400">
            <ClockCircleOutlined className="flex-shrink-0" />
            <div className="min-w-0 flex-1 truncate">
              <Tooltip title={dueAt.format(ABS_LONG)}>
                <span className="whitespace-nowrap">
                  <Text strong>{dayjs().isBefore(dueAt) ? 'Due' : 'Overdue'}</Text>{' '}
                  {dayjs().to(dueAt, true)}
                </span>
              </Tooltip>
              <span className="mx-1 text-gray-400">·</span>
              <span className="whitespace-nowrap">{fmtAbsShort(dueAt)}</span>
            </div>
          </div>
        )}

        {!openAt && !dueAt && <div className="text-gray-400">No dates</div>}
      </Space>
    </Card>
  );
};

export default AssignmentCard;
