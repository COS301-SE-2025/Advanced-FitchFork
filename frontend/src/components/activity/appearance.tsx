import type { ReactNode } from 'react';
import {
  CalendarOutlined,
  ClockCircleOutlined,
  HistoryOutlined,
  NotificationOutlined,
  UploadOutlined,
} from '@ant-design/icons';

export type ActivityAppearance = {
  label: string;
  color: string;
  icon: ReactNode;
  dotClass: string;
  titleClass: string;
};

export const getAppearance = (activityType: string): ActivityAppearance => {
  switch (activityType) {
    case 'announcement':
      return {
        label: 'Announcement',
        color: 'blue',
        icon: <NotificationOutlined className="text-lg" />,
        dotClass:
          'bg-blue-50 text-blue-600 border border-blue-200 dark:bg-blue-950/40 dark:text-blue-300 dark:border-blue-900/60',
        titleClass: 'text-blue-600 dark:text-blue-300',
      };
    case 'assignment_available':
      return {
        label: 'Assignment Available',
        color: 'purple',
        icon: <CalendarOutlined className="text-lg" />,
        dotClass:
          'bg-purple-50 text-purple-600 border border-purple-200 dark:bg-purple-950/40 dark:text-purple-300 dark:border-purple-900/60',
        titleClass: 'text-purple-600 dark:text-purple-300',
      };
    case 'assignment_due':
      return {
        label: 'Assignment Due',
        color: 'red',
        icon: <ClockCircleOutlined className="text-lg" />,
        dotClass:
          'bg-red-50 text-red-600 border border-red-200 dark:bg-red-950/40 dark:text-red-300 dark:border-red-900/60',
        titleClass: 'text-red-600 dark:text-red-300',
      };
    case 'submission':
      return {
        label: 'Submission',
        color: 'green',
        icon: <UploadOutlined className="text-lg" />,
        dotClass:
          'bg-green-50 text-green-600 border border-green-200 dark:bg-green-950/40 dark:text-green-300 dark:border-green-900/60',
        titleClass: 'text-green-600 dark:text-green-300',
      };
    default:
      return {
        label: 'Activity',
        color: 'gray',
        icon: <HistoryOutlined className="text-lg" />,
        dotClass:
          'bg-gray-100 text-gray-600 border border-gray-200 dark:bg-gray-900/50 dark:text-gray-300 dark:border-gray-800',
        titleClass: 'text-gray-700 dark:text-gray-200',
      };
  }
};
