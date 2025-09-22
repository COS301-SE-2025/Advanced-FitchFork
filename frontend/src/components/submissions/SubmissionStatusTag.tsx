import { Tag, Tooltip } from 'antd';
import { LoadingOutlined, CloseCircleOutlined, CheckCircleOutlined } from '@ant-design/icons';
import type { CSSProperties } from 'react';
import type { SubmissionStatus } from '@/types/modules/assignments/submissions';

type Props = {
  status: SubmissionStatus;
  className?: string;
  style?: CSSProperties;
  /** Optional override for the displayed text */
  labelOverride?: string;
  /** Optional tooltip text */
  tooltip?: string;
};

const LABELS: Record<SubmissionStatus, string> = {
  queued: 'Queued',
  running: 'Running',
  grading: 'Grading',
  graded: 'Graded',
  failed_upload: 'Failed: Upload',
  failed_compile: 'Failed: Compile',
  failed_execution: 'Failed: Execution',
  failed_grading: 'Failed: Grading',
  failed_internal: 'Failed: Internal',
};

const COLORS: Record<SubmissionStatus, string> = {
  queued: 'default',
  running: 'blue',
  grading: 'blue',
  graded: 'green',
  failed_upload: 'red',
  failed_compile: 'red',
  failed_execution: 'red',
  failed_grading: 'red',
  failed_internal: 'red',
};

const ICONS: Partial<Record<SubmissionStatus, React.ReactNode>> = {
  running: <LoadingOutlined />,
  graded: <CheckCircleOutlined />,
  failed_upload: <CloseCircleOutlined />,
  failed_compile: <CloseCircleOutlined />,
  failed_execution: <CloseCircleOutlined />,
  failed_grading: <CloseCircleOutlined />,
  failed_internal: <CloseCircleOutlined />,
};

export default function SubmissionStatusTag({
  status,
  className,
  style,
  labelOverride,
  tooltip,
}: Props) {
  const color = COLORS[status];
  const icon = ICONS[status];
  const label = labelOverride ?? LABELS[status];

  const tag = (
    <Tag color={color} icon={icon} className={className} style={style}>
      {label}
    </Tag>
  );

  return tooltip ? <Tooltip title={tooltip}>{tag}</Tooltip> : tag;
}
