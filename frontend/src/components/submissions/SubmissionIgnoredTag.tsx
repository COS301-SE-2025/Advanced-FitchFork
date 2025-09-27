// src/components/submissions/SubmissionIgnoredTag.tsx
import { Tag, Tooltip } from 'antd';
import { EyeInvisibleOutlined, EyeOutlined } from '@ant-design/icons';
import type { CSSProperties } from 'react';

type Props = {
  ignored?: boolean | null;
  /** Show a tag when ignored is false */
  showWhenFalse?: boolean;
  trueLabel?: string; // default: "Ignored"
  falseLabel?: string; // default: "Counted"
  tooltipTrue?: string;
  tooltipFalse?: string;
  className?: string;
  style?: CSSProperties;
};

export default function SubmissionIgnoredTag({
  ignored,
  showWhenFalse = false,
  trueLabel = 'Ignored',
  falseLabel = 'Counted',
  tooltipTrue,
  tooltipFalse,
  className,
  style,
}: Props) {
  if (ignored) {
    const tag = (
      <Tag color="magenta" icon={<EyeInvisibleOutlined />} className={className} style={style}>
        {trueLabel}
      </Tag>
    );
    return tooltipTrue ? <Tooltip title={tooltipTrue}>{tag}</Tooltip> : tag;
  }

  if (showWhenFalse) {
    const tag = (
      <Tag color="default" icon={<EyeOutlined />} className={className} style={style}>
        {falseLabel}
      </Tag>
    );
    return tooltipFalse ? <Tooltip title={tooltipFalse}>{tag}</Tooltip> : tag;
  }

  return null;
}
