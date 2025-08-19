// components/common/DateTime.tsx
import React from 'react';
import dayjs from 'dayjs';
import { ClockCircleOutlined, CalendarOutlined } from '@ant-design/icons';
import { Tooltip, Typography } from 'antd';
import { dateTimeString } from '@/utils/dateTimeString';
import type { DateTimeVariant, ValueLike } from '@/utils/dateTimeString';

export type DateTimeProps = {
  value: ValueLike;
  variant?: DateTimeVariant;
  tooltip?: boolean;
  format?: string;
  icon?: 'none' | 'calendar' | 'clock' | 'auto';
  muted?: boolean;
  className?: string;
  relativeRefreshMs?: number;
  seconds?: boolean;
  tzLabel?: string; // purely visual label (e.g. "SAST")
  tooltipFormat?: string; // optional override for tooltip (local) format
};

const DateTime: React.FC<DateTimeProps> = ({
  value,
  variant = 'datetime',
  tooltip = true,
  format,
  icon = 'auto',
  muted = false,
  className = '',
  relativeRefreshMs = 30_000,
  seconds = false,
  tzLabel,
  tooltipFormat,
}) => {
  const dt = dayjs(value);
  const isValid = dt.isValid();

  // auto-refresh for relative text
  const [, forceTick] = React.useState(0);
  React.useEffect(() => {
    if (variant !== 'relative' || relativeRefreshMs <= 0) return;
    const id = setInterval(() => forceTick((x) => x + 1), relativeRefreshMs);
    return () => clearInterval(id);
  }, [variant, relativeRefreshMs]);

  if (!isValid) {
    return <span className={`text-red-500 ${className}`}>Invalid date</span>;
  }

  // string via utility
  const text = dateTimeString(value, variant, { seconds, format });

  // icon resolution
  const withIcon =
    icon === 'auto'
      ? variant === 'time'
        ? 'clock'
        : variant === 'date'
          ? 'calendar'
          : 'none'
      : icon;

  const colorClass = muted
    ? 'text-gray-500 dark:text-gray-400'
    : 'text-gray-900 dark:text-gray-100';

  const mainNode = (
    <span className={`inline-flex items-center gap-1 ${colorClass} ${className}`}>
      {withIcon === 'calendar' && <CalendarOutlined className="opacity-70" />}
      {withIcon === 'clock' && <ClockCircleOutlined className="opacity-70" />}
      <Typography.Text className="!m-0" type={muted ? 'secondary' : undefined}>
        {text}
        {tzLabel ? <span className="ml-1 text-xs opacity-70">{tzLabel}</span> : null}
      </Typography.Text>
    </span>
  );

  if (!tooltip) return mainNode;

  // Tooltip: prefer caller's explicit format; otherwise reuse util in a long, second-precision form
  const tooltipStr =
    (tooltipFormat
      ? dayjs(value).format(tooltipFormat)
      : dateTimeString(value, 'long', { seconds: true })) + (tzLabel ? ` (${tzLabel})` : '');

  return <Tooltip title={tooltipStr}>{mainNode}</Tooltip>;
};

export default DateTime;
