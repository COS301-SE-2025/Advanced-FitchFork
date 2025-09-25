// src/components/submissions/LateTag.tsx
import { Tag, Tooltip } from 'antd';
import { FieldTimeOutlined, CheckCircleOutlined } from '@ant-design/icons';
import type { CSSProperties } from 'react';

type Props = {
  /** true -> show Late; false/undefined -> show On Time (unless showOnTime=false) */
  late?: boolean | null;

  /** If false and `late` is falsy, render nothing (hide "On Time") */
  showOnTime?: boolean;

  /** Custom labels */
  lateLabel?: string; // default: "Late"
  onTimeLabel?: string; // default: "On Time"

  /** Optional tooltips */
  tooltipLate?: string;
  tooltipOnTime?: string;

  className?: string;
  style?: CSSProperties;
};

export default function SubmissionLateTag({
  late,
  showOnTime = true,
  lateLabel,
  onTimeLabel,
  tooltipLate,
  tooltipOnTime,
  className,
  style,
}: Props) {
  if (late) {
    const tag = (
      <Tag color="red" icon={<FieldTimeOutlined />} className={className} style={style}>
        {lateLabel ?? 'Late'}
      </Tag>
    );
    return tooltipLate ? <Tooltip title={tooltipLate}>{tag}</Tooltip> : tag;
  }

  // Not late
  if (!showOnTime) return null;

  const okTag = (
    <Tag color="default" icon={<CheckCircleOutlined />} className={className} style={style}>
      {onTimeLabel ?? 'On Time'}
    </Tag>
  );
  return tooltipOnTime ? <Tooltip title={tooltipOnTime}>{okTag}</Tooltip> : okTag;
}
