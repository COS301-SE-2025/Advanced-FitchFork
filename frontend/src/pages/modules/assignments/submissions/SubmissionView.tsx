import { Typography, Tag, Descriptions, Spin, Button, Alert, Tooltip } from 'antd';
import { CodeOutlined, DownloadOutlined } from '@ant-design/icons';
import dayjs from 'dayjs';

import SubmissionTasks from '@/components/submissions/SubmissionTasks';
import { useAssignment } from '@/context/AssignmentContext';
import { useModule } from '@/context/ModuleContext';
import { useEffect, useState, useMemo } from 'react';
import { useNavigate, useParams } from 'react-router-dom';
import {
  downloadSubmissionFile,
  downloadSubmissionFileToDisk,
  getSubmissionDetails,
  getSubmissionOutput,
} from '@/services/modules/assignments/submissions';
import type {
  Submission,
  SubmissionTaskOutput,
  SubmissionStatus,
} from '@/types/modules/assignments/submissions';
import { useBreadcrumbContext } from '@/context/BreadcrumbContext';
import { message } from '@/utils/message';
import { useAuth } from '@/context/AuthContext';
import { useViewSlot } from '@/context/ViewSlotContext';
import { zipToVFiles } from '@/utils/zipToVFiles';

const { Text, Title } = Typography;

// ─────────────────────────────────────────────────────────────
// Status helpers
// ─────────────────────────────────────────────────────────────
const FAILED_STATUSES = new Set<SubmissionStatus>([
  'failed_upload',
  'failed_compile',
  'failed_execution',
  'failed_grading',
  'failed_internal',
  'failed_disallowed_code',
]);

const STATUS_META: Record<
  SubmissionStatus,
  {
    color: string;
    label: string;
    alertType?: 'error' | 'warning' | 'info' | 'success';
    blurb?: string;
  }
> = {
  queued: {
    color: 'default',
    label: 'Queued',
    alertType: 'info',
    blurb: 'Your submission is queued for processing.',
  },
  running: {
    color: 'processing',
    label: 'Running',
    alertType: 'info',
    blurb: 'We’re compiling and executing your code.',
  },
  grading: {
    color: 'gold',
    label: 'Grading',
    alertType: 'info',
    blurb: 'We’re grading your submission outputs.',
  },
  graded: { color: 'green', label: 'Graded', alertType: 'success', blurb: 'Grading complete.' },

  failed_upload: {
    color: 'red',
    label: 'Failed: Upload',
    alertType: 'error',
    blurb: 'We could not save your file. Please try uploading again.',
  },
  failed_compile: {
    color: 'red',
    label: 'Failed: Compile',
    alertType: 'error',
    blurb: 'Your code did not compile. Fix build errors and resubmit.',
  },
  failed_execution: {
    color: 'red',
    label: 'Failed: Execution',
    alertType: 'error',
    blurb: 'Your code crashed or timed out during tests.',
  },
  failed_grading: {
    color: 'red',
    label: 'Failed: Grading',
    alertType: 'error',
    blurb: 'Marking logic failed unexpectedly.',
  },
  failed_internal: {
    color: 'red',
    label: 'Failed: Internal Error',
    alertType: 'error',
    blurb: 'Something went wrong on our side.',
  },
  failed_disallowed_code: {
    color: 'volcano',
    label: 'Rejected: Disallowed Code',
    alertType: 'error',
    blurb: 'Your archive matched a disallowed pattern per policy.',
  },
};

const isFailedStatus = (s?: SubmissionStatus) => !!s && FAILED_STATUSES.has(s);

// ─────────────────────────────────────────────────────────────
// tiny helpers (dayjs)
// ─────────────────────────────────────────────────────────────
const formatLateDelta = (submittedISO?: string, dueISO?: string) => {
  if (!submittedISO || !dueISO) return null;
  const submitted = dayjs(submittedISO);
  const due = dayjs(dueISO);
  const diffMin = submitted.diff(due, 'minute');
  if (diffMin <= 0) return null;
  const h = Math.floor(diffMin / 60);
  const m = diffMin % 60;
  return h > 0 ? `+${h}h ${m}m` : `+${m}m`;
};

const minutesLate = (submittedISO?: string, dueISO?: string): number | null => {
  if (!submittedISO || !dueISO) return null;
  const diffMin = dayjs(submittedISO).diff(dayjs(dueISO), 'minute');
  return diffMin <= 0 ? 0 : diffMin;
};

const SubmissionView = () => {
  const auth = useAuth();
  const navigate = useNavigate();
  const [submission, setSubmission] = useState<Submission | null>(null);
  const [submissionOutput, setSubmissionOutput] = useState<SubmissionTaskOutput[]>([]);
  const [loading, setLoading] = useState(true);

  const module = useModule();
  const { assignment, memoOutput, policy, config } = useAssignment();

  const { submission_id } = useParams();
  const submissionId = Number(submission_id);
  const { setBreadcrumbLabel } = useBreadcrumbContext();
  const { setValue } = useViewSlot();

  useEffect(() => {
    setValue(
      <Typography.Text className="text-base font-medium text-gray-900 dark:text-gray-100 truncate">
        Submission {submission?.id}
      </Typography.Text>,
    );
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [submission?.id]);

  const fetchSubmission = async () => {
    if (!module.id || !assignment.id || !submission_id) return;
    setLoading(true);
    try {
      const res = await getSubmissionDetails(module.id, assignment.id, Number(submission_id));
      if (res.success && res.data) {
        setSubmission(res.data);
        setBreadcrumbLabel(
          `modules/${module.id}/assignments/${assignment.id}/submissions/${res.data.id}`,
          `Attempt #${res.data.attempt}`,
        );
      }
    } catch (err) {
      console.error('Failed to load submission details', err);
    } finally {
      setLoading(false);
    }
  };

  const fetchSubmisisonOutput = async () => {
    try {
      const res = await getSubmissionOutput(module.id, assignment.id, submissionId);
      if (res.success && res.data) setSubmissionOutput(res.data);
      else message.error(res.message);
    } catch {
      message.error('Failed to fetch submission output');
    }
  };

  const handleViewInIDE = async () => {
    if (!module.id || !assignment.id || !submissionId) return;
    try {
      const blob = await downloadSubmissionFile(module.id, assignment.id, submissionId);
      const arrayBuffer = await blob.arrayBuffer();
      const files = await zipToVFiles(arrayBuffer);
      navigate(
        `/modules/${module.id}/assignments/${assignment.id}/submissions/${submissionId}/code`,
        { state: { files } },
      );
    } catch (err) {
      console.error('Failed to open submission in IDE', err);
      message.error('Could not open submission in IDE');
    }
  };

  const handleDownload = async () => {
    if (!module.id || !assignment.id || !submissionId) return;
    try {
      await downloadSubmissionFileToDisk(module.id, assignment.id, submissionId);
    } catch {
      message.error('Download failed');
    }
  };

  useEffect(() => {
    fetchSubmission();
    if (!auth.isStudent(module.id)) fetchSubmisisonOutput();
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [module.id, assignment.id, submissionId]);

  // derived late info (used only for non-failed)
  const lateDelta = useMemo(
    () => formatLateDelta(submission?.created_at, assignment?.due_date),
    [submission?.created_at, assignment?.due_date],
  );
  const minsLate = useMemo(
    () => minutesLate(submission?.created_at, assignment?.due_date),
    [submission?.created_at, assignment?.due_date],
  );

  if (loading || !submission) {
    return (
      <div className="p-6 max-w-4xl">
        <Spin />
      </div>
    );
  }

  const { mark, attempt, created_at, hash, tasks, status } = submission;

  const statusMeta = STATUS_META[status];
  const failed = isFailedStatus(status);

  // special failed view: single column, no tasks panel, no right-hand descriptions
  if (failed) {
    const canDownload = status !== 'failed_upload';
    const canOpenIDE = canDownload;

    return (
      <div className=" !space-y-4">
        <Alert
          className="bg-white dark:bg-gray-900"
          type={statusMeta?.alertType || 'error'}
          showIcon
          message={
            <div className="flex flex-wrap items-center gap-2">
              <Tag color={statusMeta?.color || 'red'}>{statusMeta?.label || 'Failed'}</Tag>
              <Text className="font-medium">{statusMeta?.blurb || 'This submission failed.'}</Text>
            </div>
          }
        />

        <div className="bg-white dark:bg-gray-900 border border-gray-200 dark:border-gray-800 rounded-md p-5 !space-y-4">
          <div className="flex items-center justify-between flex-wrap gap-3">
            <Title level={4} className="!mb-0">
              Submission #{submission.id}
            </Title>
            <div className="flex items-center gap-2">
              <Tooltip title={canDownload ? undefined : 'No file was saved for this submission'}>
                <Button
                  icon={<DownloadOutlined />}
                  onClick={handleDownload}
                  disabled={!canDownload}
                >
                  Download File
                </Button>
              </Tooltip>
              <Tooltip title={canOpenIDE ? undefined : 'No file available to open'}>
                <Button icon={<CodeOutlined />} onClick={handleViewInIDE} disabled={!canOpenIDE}>
                  View in IDE
                </Button>
              </Tooltip>
            </div>
          </div>

          <Descriptions bordered column={2} size="middle" className="bg-white dark:bg-gray-900">
            <Descriptions.Item label="Attempt">
              <Tag>#{attempt}</Tag>
            </Descriptions.Item>
            <Descriptions.Item label="Uploaded At">
              {created_at ? dayjs(created_at).format('YYYY-MM-DD HH:mm') : '—'}
            </Descriptions.Item>
            <Descriptions.Item label="Status" span={2}>
              <Tag color={statusMeta?.color || 'red'}>{statusMeta?.label || status}</Tag>
            </Descriptions.Item>
            <Descriptions.Item label="File Hash (MD5)" span={2}>
              <Text code>{hash || '—'}</Text>
            </Descriptions.Item>
          </Descriptions>

          <Alert
            type="info"
            showIcon
            className="bg-white dark:bg-gray-900"
            message="What can I do?"
            description={
              <ul className="list-disc ml-5 space-y-1">
                {status === 'failed_compile' && (
                  <li>Open your project locally and fix build errors, then resubmit.</li>
                )}
                {status === 'failed_execution' && (
                  <li>Check runtime errors/timeouts in your tests and try again.</li>
                )}
                {status === 'failed_upload' && (
                  <li>
                    Upload the file again. Ensure it’s a supported archive (.zip/.tgz/.gz/.tar).
                  </li>
                )}
                {status === 'failed_disallowed_code' && (
                  <li>
                    Remove disallowed content (e.g., forbidden libraries or code patterns) and
                    resubmit according to the assignment policy.
                  </li>
                )}
                {(status === 'failed_grading' || status === 'failed_internal') && (
                  <li>
                    If this persists, contact the teaching team and include your submission ID #
                    {submission.id}.
                  </li>
                )}
              </ul>
            }
          />
        </div>
      </div>
    );
  }

  // ─────────────────────────────────────────────────────────────
  // Normal (non-failed) view: original layout
  // ─────────────────────────────────────────────────────────────

  const isLate = !!submission.is_late;
  const allowLate = !!(policy?.late ?? config?.marking?.late)?.allow_late_submissions;
  const lateWindow = (policy?.late ?? config?.marking?.late)?.late_window_minutes ?? 0;
  const lateMaxPercent = (policy?.late ?? config?.marking?.late)?.late_max_percent ?? 100;

  const total = submission.mark?.total ?? 0;
  const capMax = total ? Math.round((lateMaxPercent * total) / 100) : null;
  const withinWindow = typeof minsLate === 'number' ? minsLate <= lateWindow : true;

  const percentage = mark?.total ? Math.round((mark.earned / mark.total) * 100) : null;
  const markColor =
    percentage === null
      ? 'default'
      : percentage >= 75
        ? 'green'
        : percentage >= 50
          ? 'orange'
          : 'red';

  const canDownload = true; // normal flow allows download
  const canOpenIDE = true;

  return (
    <div className="flex flex-col lg:flex-row gap-4 pb-4">
      {/* Left: Tasks */}
      <div className="order-2 lg:order-1 lg:w-2/3 !space-y-4">
        {/* Status banner (non-failed) */}
        {statusMeta?.alertType && (
          <Alert
            className="bg-white dark:bg-gray-900"
            type={statusMeta.alertType}
            showIcon
            message={
              <div className="flex items-center gap-2">
                <Tag color={statusMeta.color}>{statusMeta.label}</Tag>
                <Text className="font-medium">{statusMeta.blurb}</Text>
              </div>
            }
          />
        )}

        <div className="bg-white dark:bg-gray-900 border border-gray-200 dark:border-gray-800 rounded-md p-4">
          <Title level={4} className="!mb-0">
            Tasks
          </Title>
        </div>

        {isLate && (
          <Alert
            type={!allowLate ? 'error' : withinWindow ? 'warning' : 'error'}
            showIcon
            className="bg-white dark:bg-gray-900"
            message={
              <div className="flex flex-wrap items-center gap-8">
                <div className="flex items-center gap-2">
                  <Tag color="red">Late</Tag>
                  <Text className="font-medium">
                    {lateDelta
                      ? `Submitted ${lateDelta} after the deadline`
                      : 'Submitted after the deadline'}
                  </Text>
                </div>

                <div className="flex items-center gap-2">
                  {allowLate && withinWindow && <Tag>Within window</Tag>}
                  {allowLate && !withinWindow && <Tag color="volcano">Outside window</Tag>}
                  {allowLate && total > 0 && (
                    <Tooltip
                      title={`Late cap: up to ${lateMaxPercent}% of total${capMax ? ` (max ${capMax}/${total})` : ''}`}
                    >
                      <Tag color="volcano">Cap {lateMaxPercent}%</Tag>
                    </Tooltip>
                  )}
                </div>
              </div>
            }
          />
        )}

        <SubmissionTasks
          tasks={tasks ?? []}
          memoOutput={memoOutput}
          submisisonOutput={submissionOutput}
          codeCoverage={submission.code_coverage}
        />
      </div>

      {/* Right: Description */}
      <div className="order-1 lg:order-2 lg:w-1/3 space-y-6">
        <Descriptions bordered column={1} size="middle" className="bg-white dark:bg-gray-900">
          {/* Status */}
          <Descriptions.Item label="Status">
            <Tag color={STATUS_META[status]?.color || 'default'}>
              {STATUS_META[status]?.label || status}
            </Tag>
          </Descriptions.Item>

          {submission.user && (
            <>
              <Descriptions.Item label="Username">{submission.user.username}</Descriptions.Item>
              <Descriptions.Item label="Email">{submission.user.email}</Descriptions.Item>
            </>
          )}

          <Descriptions.Item label="Mark">
            {mark ? (
              <>
                <Tag color={markColor}>
                  {mark.earned} / {mark.total}
                </Tag>
                <Text type="secondary"> ({percentage}%)</Text>
                {isLate && allowLate && (
                  <>
                    <span className="mx-2">•</span>
                    <Tooltip
                      title={`If earned > ${lateMaxPercent}% of total, the awarded mark is capped${
                        capMax && total ? ` (max ${capMax}/${total})` : ''
                      }.`}
                    >
                      <Tag color="volcano">Late cap</Tag>
                    </Tooltip>
                  </>
                )}
              </>
            ) : (
              <Tag color="default">Not graded</Tag>
            )}
          </Descriptions.Item>

          <Descriptions.Item label="Attempt">
            <Tag>#{attempt}</Tag>
          </Descriptions.Item>

          <Descriptions.Item label="Uploaded At">
            <div className="flex items-center gap-2">
              <span>{created_at ? dayjs(created_at).format('YYYY-MM-DD HH:mm') : '—'}</span>
              {submission.is_late && (
                <Tooltip title={`Due: ${dayjs(assignment.due_date).format('YYYY-MM-DD HH:mm')}`}>
                  <Tag color="red">Late{lateDelta ? ` ${lateDelta}` : ''}</Tag>
                </Tooltip>
              )}
            </div>
          </Descriptions.Item>

          <Descriptions.Item label="File Hash (MD5)">
            <Text code>{hash || '—'}</Text>
          </Descriptions.Item>

          <Descriptions.Item label="File">
            <Button
              icon={<DownloadOutlined />}
              type="link"
              size="small"
              onClick={handleDownload}
              disabled={!canDownload}
            >
              Download File
            </Button>
            <Button
              icon={<CodeOutlined />}
              type="link"
              size="small"
              onClick={handleViewInIDE}
              disabled={!canOpenIDE}
            >
              View in IDE
            </Button>
          </Descriptions.Item>
        </Descriptions>
      </div>
    </div>
  );
};

export default SubmissionView;
