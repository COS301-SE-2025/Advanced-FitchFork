import { Card, Space, Tag, Typography } from 'antd';
import { CalendarOutlined, ClockCircleOutlined, FileTextOutlined } from '@ant-design/icons';
import type { Assignment } from '@/types/assignments';

import dayjs from 'dayjs';
import relativeTime from 'dayjs/plugin/relativeTime';
dayjs.extend(relativeTime);

const { Text, Paragraph } = Typography;

interface Props {
  assignment: Assignment;
}

const AssignmentCard = ({ assignment }: Props) => {
  return (
    <Card
      className="dark:bg-neutral-900 dark:border-none hover:shadow-lg transition-shadow duration-200"
      title={
        <div className="flex justify-between items-start">
          <div className="flex items-center gap-2">
            <FileTextOutlined className="text-lg text-blue-500" />
            <span className="font-medium text-base">{assignment.name}</span>
          </div>
          <Tag color="blue" className="text-xs uppercase">
            {assignment.assignment_type}
          </Tag>
        </div>
      }
    >
      <Paragraph className="text-sm text-gray-700 dark:text-gray-300 mb-4">
        {assignment.description || 'No description provided.'}
      </Paragraph>

      <Space direction="vertical" size={4} className="text-sm">
        <div className="flex items-center gap-2 text-gray-500 dark:text-gray-400">
          <CalendarOutlined />
          <span>
            <Text strong className="mr-1">
              Opens:
            </Text>
            {dayjs(assignment.available_from).format('MMM D, YYYY · h:mm A')} (
            {dayjs(assignment.available_from).fromNow()})
          </span>
        </div>
        <div className="flex items-center gap-2 text-gray-500 dark:text-gray-400">
          <ClockCircleOutlined />
          <span>
            <Text strong className="mr-1">
              Closes:
            </Text>
            {dayjs(assignment.due_date).format('MMM D, YYYY · h:mm A')} (
            {dayjs(assignment.due_date).fromNow()})
          </span>
        </div>
      </Space>
    </Card>
  );
};

export default AssignmentCard;
