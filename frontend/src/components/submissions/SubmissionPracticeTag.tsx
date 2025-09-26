// src/components/submissions/SubmissionPracticeTag.tsx
import { Tag, Tooltip } from 'antd';
import type { CSSProperties } from 'react';

type Props = {
  practice?: boolean | null;
  /** Show a tag when practice is false */
  showWhenFalse?: boolean;
  trueLabel?: string; // default: "Practice"
  falseLabel?: string; // default: "Official"
  tooltipTrue?: string;
  tooltipFalse?: string;
  className?: string;
  style?: CSSProperties;
};

export default function SubmissionPracticeTag({
  practice,
  showWhenFalse = false,
  trueLabel = 'Practice',
  falseLabel = 'Official',
  tooltipTrue,
  tooltipFalse,
  className,
  style,
}: Props) {
  if (practice) {
    const tag = (
      <Tag color="gold" className={className} style={style}>
        {trueLabel}
      </Tag>
    );
    return tooltipTrue ? <Tooltip title={tooltipTrue}>{tag}</Tooltip> : tag;
  }

  if (showWhenFalse) {
    const tag = (
      <Tag color="default" className={className} style={style}>
        {falseLabel}
      </Tag>
    );
    return tooltipFalse ? <Tooltip title={tooltipFalse}>{tag}</Tooltip> : tag;
  }

  return null;
}
