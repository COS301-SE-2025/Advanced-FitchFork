import type { TimelineProps } from 'antd';

import type { ActivityItem } from '@/types/me/activity';

import ActivityTimelineItemCard from './ActivityTimelineItemCard';
import { getAppearance } from './appearance';

export const createTimelineItem = (
  activity: ActivityItem,
): NonNullable<TimelineProps['items']>[number] => {
  const appearance = getAppearance(activity.activity_type);

  return {
    key: activity.id,
    color: appearance.color,
    dot: (
      <span
        className={`flex h-8 w-8 items-center justify-center rounded-full ${appearance.dotClass}`}
        style={{ color: 'currentColor' }}
      >
        {appearance.icon}
      </span>
    ),
    children: <ActivityTimelineItemCard activity={activity} appearance={appearance} />,
  };
};
