import { Timeline, Typography } from 'antd';
import type { TimelineProps } from 'antd';

interface Props {
  title: string;
  items: TimelineProps['items'];
}

const ActivitySection = ({ title, items }: Props) => {
  if (!items || items.length === 0) {
    return null;
  }

  return (
    <section className="!space-y-3">
      <Typography.Title level={4} className=" text-gray-900 dark:text-gray-100">
        {title}
      </Typography.Title>
      <div className="!pt-1 !pl-4">
        <Timeline mode="left" items={items} className="activity-timeline" />
      </div>
    </section>
  );
};

export default ActivitySection;
