import { Statistic } from 'antd';
import type { StatisticProps } from 'antd';
import clsx from 'clsx';

export interface StatCardProps extends StatisticProps {
  className?: string; // Wrapper card styling
}

export default function StatCard({
  title,
  value,
  suffix,
  prefix,
  className,
  ...rest
}: StatCardProps) {
  return (
    <div
      className={clsx(
        'bg-white dark:bg-gray-950 p-4 rounded-lg border border-gray-200 dark:border-gray-700',
        className,
      )}
    >
      <div className="flex items-center space-x-3">
        <div className="flex-1">
          <Statistic
            title={<span className="text-sm text-gray-500">{title}</span>}
            value={value}
            suffix={suffix}
            {...rest}
            prefix={prefix}
          />
        </div>
      </div>
    </div>
  );
}
