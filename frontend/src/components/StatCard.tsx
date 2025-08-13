import { Statistic } from 'antd';
import type { StatisticProps } from 'antd';
import clsx from 'clsx';

export interface StatCardProps extends StatisticProps {
  className?: string;
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
        'bg-white dark:bg-gray-900 p-4 rounded-lg border border-gray-200 dark:border-gray-700 h-full',
        className,
      )}
    >
      <div className="flex items-start gap-3 min-w-0">
        <div className="shrink-0">{prefix}</div>
        <div className="flex-1 min-w-0">
          <Statistic
            title={
              <span className="text-sm text-gray-500 whitespace-normal break-words">{title}</span>
            }
            value={String(value)}
            suffix={suffix}
            {...rest}
          />
        </div>
      </div>
    </div>
  );
}
