import React from 'react';
import { Collapse, Tag, Typography } from 'antd';
import { CheckCircleOutlined, CloseCircleOutlined, InfoCircleOutlined } from '@ant-design/icons';

const { Text } = Typography;

export type SectionFeedback = {
  label: string;
  score: string;
  status: 'correct' | 'incorrect' | 'info';
};

export type SubmissionTask = {
  key: string;
  title: string;
  score: string;
  feedback: SectionFeedback[];
};

type Props = {
  tasks: SubmissionTask[];
};

const getStatusColor = (status: SectionFeedback['status']) => {
  switch (status) {
    case 'correct':
      return 'green';
    case 'incorrect':
      return 'red';
    case 'info':
    default:
      return 'blue';
  }
};

const getStatusIcon = (status: SectionFeedback['status']) => {
  const iconSize = 18; // You can tweak this size (default is 14)
  switch (status) {
    case 'correct':
      return <CheckCircleOutlined style={{ fontSize: iconSize, color: '#52c41a' }} />;
    case 'incorrect':
      return <CloseCircleOutlined style={{ fontSize: iconSize, color: '#ff4d4f' }} />;
    case 'info':
    default:
      return <InfoCircleOutlined style={{ fontSize: iconSize, color: '#1890ff' }} />;
  }
};

const getScoreTagColor = (score: string): string => {
  const [got, total] = score.split('/').map((s) => parseFloat(s.trim()));
  if (isNaN(got) || isNaN(total) || total === 0) return 'default';
  const percent = (got / total) * 100;
  if (percent >= 85) return 'green';
  if (percent <= 50) return 'red';
  return 'orange';
};

const SubmissionTasks: React.FC<Props> = ({ tasks }) => {
  return (
    <Collapse
      bordered={false}
      className="!bg-white dark:!bg-gray-900 !border !border-gray-200 dark:!border-gray-700"
      expandIconPosition="end"
    >
      {tasks.map((task) => (
        <Collapse.Panel
          key={task.key}
          header={
            <div className="flex justify-between items-center">
              <Text className="text-base font-medium">{task.title}</Text>
              <Tag color={getScoreTagColor(task.score)}>{task.score}</Tag>
            </div>
          }
        >
          <ul className="space-y-2 pl-2">
            {task.feedback.map((fb, idx) => (
              <li
                key={idx}
                className="flex items-center gap-2 text-sm text-neutral-700 dark:text-neutral-300"
              >
                {getStatusIcon(fb.status)}
                <span className="flex-1">{fb.label}</span>
                <Tag color={getStatusColor(fb.status)}>{fb.score}</Tag>
              </li>
            ))}
          </ul>
        </Collapse.Panel>
      ))}
    </Collapse>
  );
};

export default SubmissionTasks;
