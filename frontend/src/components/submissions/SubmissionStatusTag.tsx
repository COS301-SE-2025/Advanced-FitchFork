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
  /** Optional tooltip text (overrides the default blurb) */
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
  failed_disallowed_code: 'Rejected: Disallowed Code',
};

const COLORS: Record<SubmissionStatus, string> = {
  // AntD Tag preset colors: 'default' | 'processing' | 'success' | 'error' | 'warning'
  // plus palette colors like 'blue', 'gold', 'green', 'volcano', etc.
  queued: 'default',
  running: 'blue',
  grading: 'gold',
  graded: 'green',
  failed_upload: 'red',
  failed_compile: 'red',
  failed_execution: 'red',
  failed_grading: 'red',
  failed_internal: 'red',
  failed_disallowed_code: 'volcano',
};

const ICONS: Partial<Record<SubmissionStatus, React.ReactNode>> = {
  queued: <LoadingOutlined />,
  running: <LoadingOutlined />,
  graded: <CheckCircleOutlined />,
  failed_upload: <CloseCircleOutlined />,
  failed_compile: <CloseCircleOutlined />,
  failed_execution: <CloseCircleOutlined />,
  failed_grading: <CloseCircleOutlined />,
  failed_internal: <CloseCircleOutlined />,
  failed_disallowed_code: <CloseCircleOutlined />,
};

const BLURBS: Partial<Record<SubmissionStatus, string>> = {
  queued: 'Your submission is queued for processing.',
  running: 'Compiling and executing your code.',
  grading: 'Marking your submission outputs.',
  graded: 'Grading complete.',
  failed_upload: 'We could not save your file. Try uploading again.',
  failed_compile: 'Your code did not compile. Fix build errors and resubmit.',
  failed_execution: 'Your code crashed or timed out during tests.',
  failed_grading: 'Marking logic failed unexpectedly.',
  failed_internal: 'An unexpected internal error occurred.',
  failed_disallowed_code: 'Archive matched a disallowed pattern per assignment policy.',
};

export default function SubmissionStatusTag({
  status,
  className,
  style,
  labelOverride,
  tooltip,
}: Props) {
  const color = COLORS[status] || 'default';
  const icon = ICONS[status];
  const label = labelOverride ?? LABELS[status];
  const tip = tooltip ?? BLURBS[status];

  const tag = (
    <Tag color={color} icon={icon} className={className} style={style}>
      {label}
    </Tag>
  );

  return tip ? <Tooltip title={tip}>{tag}</Tooltip> : tag;
}
