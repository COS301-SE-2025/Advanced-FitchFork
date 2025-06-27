import React from 'react';
import { Collapse, Tag, Typography } from 'antd';
import { CheckCircleOutlined, CloseCircleOutlined, InfoCircleOutlined } from '@ant-design/icons';
import type { TaskBreakdown } from '@/types/modules/assignments/submissions';

const { Text } = Typography;

type Props = {
  tasks: TaskBreakdown[];
};

const getStatusColor = (status?: string): string => {
  switch (status?.toLowerCase()) {
    case 'correct':
      return 'green';
    case 'incorrect':
      return 'red';
    case 'info':
      return 'blue';
    default:
      return 'default';
  }
};

const getStatusIcon = (status?: string) => {
  const iconSize = 18;
  switch (status?.toLowerCase()) {
    case 'correct':
      return <CheckCircleOutlined style={{ fontSize: iconSize, color: '#52c41a' }} />;
    case 'incorrect':
      return <CloseCircleOutlined style={{ fontSize: iconSize, color: '#ff4d4f' }} />;
    case 'info':
      return <InfoCircleOutlined style={{ fontSize: iconSize, color: '#1890ff' }} />;
    default:
      return <InfoCircleOutlined style={{ fontSize: iconSize, color: '#d9d9d9' }} />;
  }
};

const getScoreTagColor = (earned: number, total: number): string => {
  if (total === 0) return 'default';
  const percent = (earned / total) * 100;
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
      {tasks.map((task) => {
        const { task_number, name, score, subsections } = task;
        return (
          <Collapse.Panel
            key={task_number}
            header={
              <div className="flex justify-between items-center">
                <Text className="text-base font-medium">{name}</Text>
                <Tag color={getScoreTagColor(score.earned, score.total)}>
                  {score.earned} / {score.total}
                </Tag>
              </div>
            }
          >
            <ul className="space-y-2 pl-2">
              {subsections.map((sub, idx) => (
                <li
                  key={idx}
                  className="flex items-center gap-2 text-sm text-neutral-700 dark:text-neutral-300"
                >
                  {getStatusIcon(sub.status)}
                  <span className="flex-1">{sub.label}</span>
                  <Tag color={getStatusColor(sub.status)}>
                    {sub.earned} / {sub.total}
                  </Tag>
                </li>
              ))}
            </ul>
          </Collapse.Panel>
        );
      })}
    </Collapse>
  );
};

export default SubmissionTasks;
