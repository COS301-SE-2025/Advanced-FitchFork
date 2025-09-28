// src/components/submissions/AttemptTag.tsx
import { Tag, Tooltip } from 'antd';
import { NumberOutlined, FieldNumberOutlined } from '@ant-design/icons';
import type { CSSProperties, ReactNode } from 'react';

type Mode = 'number' | 'label';

type Props = {
  /** Attempt number to display */
  attempt: number;
  /** 'number' → just the number (e.g. "#3"), 'label' → "Attempt #3" */
  mode?: Mode;
  /** When mode='label', customize the label text (defaults to "Attempt") */
  labelText?: string;
  /** Include a leading '#' (defaults to true in both modes) */
  hash?: boolean;
  /** Optional tooltip */
  tooltip?: string;
  /** Customize color (defaults to 'blue') */
  color?: string;
  /** Optional icon override */
  icon?: ReactNode;
  className?: string;
  style?: CSSProperties;
};

export default function SubmissionAttemptTag({
  attempt,
  mode = 'label',
  labelText = 'Attempt',
  hash = true,
  tooltip,
  color = 'blue',
  icon,
  className,
  style,
}: Props) {
  const hashStr = hash ? `#${attempt}` : String(attempt);
  const content = mode === 'label' ? `${labelText} ${hashStr}` : hashStr;

  const tag = (
    <Tag
      color={color}
      className={className}
      style={style}
      icon={icon ?? (mode === 'label' ? <NumberOutlined /> : <FieldNumberOutlined />)}
    >
      {content}
    </Tag>
  );

  return tooltip ? <Tooltip title={tooltip}>{tag}</Tooltip> : tag;
}
