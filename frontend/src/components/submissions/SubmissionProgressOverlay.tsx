import { useEffect, useRef, useState } from 'react';
import { useNavigate } from 'react-router-dom';
import { Modal, Result, Space, Typography, Button, Progress, Divider } from 'antd';
import {
  CheckCircleTwoTone,
  ClockCircleTwoTone,
  CloseCircleTwoTone,
  LoadingOutlined,
} from '@ant-design/icons';
import confetti, { type Options as ConfettiOptions } from 'canvas-confetti';

import { scaleColor } from '@/utils/color';
import {
  SUBMISSION_STATUSES,
  type SubmissionStatus,
} from '@/types/modules/assignments/submissions';
import { useWs, useWsEvents, Topics, type SubmissionStatusPayload } from '@/ws';

const { Title } = Typography;

type Props = {
  moduleId: number;
  assignmentId: number;
  userId: number;
  /** Called when modal is closed */
  onClose?: () => void;
  /** Called when submission is done and user clicks “Go to submission” */
  onDone?: (submissionId: number) => void;
  submissionId?: number;
  triggerSubmit?: () => Promise<number | null>;
  wsConnectTimeoutMs?: number;
};

const STATUS_PROGRESS: Record<SubmissionStatus, number> = {
  queued: 10,
  running: 40,
  grading: 80,
  graded: 100,
  failed_upload: 100,
  failed_compile: 100,
  failed_execution: 100,
  failed_grading: 100,
  failed_internal: 100,
  failed_disallowed_code: 100,
};

function statusTitle(status: SubmissionStatus | 'queued' | 'connecting') {
  switch (status) {
    case 'connecting':
      return 'Connecting live updates…';
    case 'queued':
      return 'Queued — waiting to start…';
    case 'running':
      return 'Running tests…';
    case 'grading':
      return 'Grading…';
    case 'graded':
      return 'Finished';
    case 'failed_upload':
      return 'Upload failed';
    case 'failed_compile':
      return 'Compilation failed';
    case 'failed_execution':
      return 'Execution failed';
    case 'failed_grading':
      return 'Grading failed';
    case 'failed_internal':
      return 'Internal error';
    default:
      return 'Processing your submission…';
  }
}

function statusSubtitle(status: SubmissionStatus | 'queued' | 'connecting') {
  switch (status) {
    case 'connecting':
      return 'Waiting for the live WebSocket connection.';
    case 'queued':
      return 'Your job is in the queue.';
    case 'running':
      return 'We’re executing your code and capturing outputs.';
    case 'grading':
      return 'We’re marking your results.';
    default:
      return 'This view updates live.';
  }
}

type Latest = {
  submissionId: number;
  status: SubmissionStatus;
  mark?: { earned: number; total: number } | null;
  message?: string | null;
  ts?: string | null;
};

export default function SubmissionProgressOverlay({
  moduleId,
  assignmentId,
  userId,
  onClose,
  onDone,
  submissionId,
  triggerSubmit,
  wsConnectTimeoutMs,
}: Props) {
  const navigate = useNavigate();

  // WS connection status (from context)
  const { status: wsStatus } = useWs();
  const connected = wsStatus === 'open';

  // Latest owner event snapshot
  const [latest, setLatest] = useState<Latest | null>(null);

  // Subscribe to the *owner* stream for this user
  useWsEvents([Topics.assignmentSubmissionsOwner(assignmentId, userId)], {
    'submission.status': (p: SubmissionStatusPayload) => {
      if (p.assignment_id !== assignmentId) return; // safety
      const m =
        p.mark && typeof p.mark.total === 'number'
          ? { earned: Number(p.mark.earned), total: Number(p.mark.total) }
          : null;
      setLatest({
        submissionId: Number(p.submission_id),
        status: p.status as SubmissionStatus,
        mark: m,
        message: p.message ?? null,
        ts: (p as any)?.ts ?? null,
      });
    },
  });

  // ---- Defer submit: fire on WS connect (once), or fallback after timeout ----
  const firedRef = useRef(false);
  useEffect(() => {
    if (!triggerSubmit || firedRef.current) return;

    const timeoutMs = Math.max(500, wsConnectTimeoutMs ?? 2000);
    const timer = window.setTimeout(() => {
      if (!firedRef.current) {
        firedRef.current = true;
        void triggerSubmit();
      }
    }, timeoutMs) as unknown as number;

    if (connected && !firedRef.current) {
      Promise.resolve().then(() => {
        if (!firedRef.current) {
          firedRef.current = true;
          void triggerSubmit();
        }
      });
    }

    return () => {
      window.clearTimeout(timer);
    };
  }, [connected, triggerSubmit, wsConnectTimeoutMs]);

  // Active progress (WS only)
  const progress = latest ?? null;
  const activeSubmissionId = submissionId ?? progress?.submissionId ?? null;

  const isSubmissionStatus = (v: unknown): v is SubmissionStatus =>
    typeof v === 'string' && (SUBMISSION_STATUSES as readonly string[]).includes(v);

  const rawStatus: SubmissionStatus | 'queued' | 'connecting' =
    progress?.status && isSubmissionStatus(progress.status)
      ? progress.status
      : connected
        ? 'queued'
        : 'connecting';

  const isFailed = rawStatus.startsWith('failed_');
  const isGraded = rawStatus === 'graded';
  const linearPercent =
    rawStatus === 'connecting'
      ? 5
      : isSubmissionStatus(rawStatus)
        ? STATUS_PROGRESS[rawStatus]
        : 10;

  const mark = progress?.mark ?? undefined;
  const pct = mark && mark.total > 0 ? Math.round((mark.earned / mark.total) * 100) : null;
  const circleColor = pct != null ? scaleColor(pct, 'red-green') : undefined;

  const handleClose = () => {
    onClose?.();
  };

  const handleGoToSubmission = () => {
    if (!activeSubmissionId) return;
    onDone?.(activeSubmissionId);
    navigate(`/modules/${moduleId}/assignments/${assignmentId}/submissions/${activeSubmissionId}`);
    onClose?.();
  };

  // ===== Confetti =====
  const prevStatusRef = useRef<SubmissionStatus | 'queued' | 'connecting' | null>(null);
  const firedForSubmissionRef = useRef<number | null>(null);

  const launcherRef = useRef<ReturnType<typeof confetti.create> | null>(null);
  useEffect(() => {
    if (!launcherRef.current) {
      launcherRef.current = confetti.create(undefined, { resize: true, useWorker: false });
    }
    return () => {
      try {
        (launcherRef.current as any)?.reset?.();
      } catch {}
    };
  }, []);

  useEffect(() => {
    const launch = launcherRef.current ?? confetti;

    const justGraded = prevStatusRef.current !== 'graded' && isGraded;
    const isPerfect = pct === 100 || (!!mark && mark.total > 0 && mark.earned === mark.total);
    const alreadyFiredForThis =
      firedForSubmissionRef.current != null &&
      activeSubmissionId != null &&
      firedForSubmissionRef.current === activeSubmissionId;

    const shouldFire =
      justGraded && isPerfect && !alreadyFiredForThis && activeSubmissionId != null;

    if (shouldFire) {
      firedForSubmissionRef.current = activeSubmissionId;
      const shot = (particleRatio: number, opts: ConfettiOptions = {}) =>
        launch({
          origin: { y: 0.3 },
          spread: 70,
          startVelocity: 55,
          ticks: 200,
          scalar: 1,
          zIndex: 2147483647,
          disableForReducedMotion: false,
          ...opts,
          particleCount: Math.floor(300 * particleRatio),
        });
      shot(0.25, { angle: 60, origin: { x: 0, y: 0.4 } });
      shot(0.25, { angle: 120, origin: { x: 1, y: 0.4 } });
      shot(0.5, { spread: 100, origin: { x: 0.5, y: 0.3 } });
    }

    prevStatusRef.current = rawStatus;
  }, [rawStatus, pct, isGraded, activeSubmissionId, mark]);

  // ===== Body content =====
  const body = (
    <Space direction="vertical" size={16} style={{ width: '100%' }}>
      {!isGraded && (
        <>
          <Progress
            percent={linearPercent}
            status={isFailed ? 'exception' : rawStatus === 'connecting' ? 'normal' : 'active'}
          />
          <Divider style={{ margin: '8px 0' }} />
        </>
      )}

      {isFailed && (
        <Result
          status="error"
          title="Submission Failed"
          subTitle={
            typeof (progress as any)?.message === 'string'
              ? (progress as any).message
              : 'Please check your build/run logs.'
          }
        />
      )}

      {!isFailed && !isGraded && (
        <Result
          icon={<LoadingOutlined style={{ fontSize: 40 }} spin />}
          title={statusTitle(rawStatus)}
          subTitle={statusSubtitle(rawStatus)}
        />
      )}

      {isGraded && mark && (
        <div className="flex flex-col items-center justify-center gap-4 py-2">
          <Progress
            type="circle"
            percent={pct ?? 0}
            size={220}
            strokeColor={circleColor}
            trailColor="rgba(0,0,0,0.08)"
            format={() => (
              <div className="flex flex-col items-center justify-center">
                <div className="text-base font-semibold">
                  {mark.earned} / {mark.total}
                </div>
                <div className="text-sm" style={{ color: circleColor }}>
                  {pct}%
                </div>
              </div>
            )}
          />
          <Title level={3} className="!m-0 !text-center">
            Submission Graded
          </Title>
        </div>
      )}
    </Space>
  );

  return (
    <Modal
      open
      closable={false}
      maskClosable={false}
      keyboard={false}
      width={720}
      centered
      destroyOnHidden
      footer={
        <div style={{ display: 'flex', justifyContent: 'center', gap: 12 }}>
          <Button onClick={handleClose}>Close</Button>
          <Button
            type="primary"
            onClick={handleGoToSubmission}
            disabled={!activeSubmissionId || (!isFailed && !isGraded)}
          >
            Go to submission
          </Button>
        </div>
      }
      title={
        isGraded ? null : (
          <div className="flex items-center justify-center gap-2">
            {isFailed ? (
              <CloseCircleTwoTone twoToneColor="#ff4d4f" />
            ) : isGraded ? (
              <CheckCircleTwoTone twoToneColor="#52c41a" />
            ) : (
              <ClockCircleTwoTone twoToneColor="#1677ff" />
            )}
            <Title level={4} className="!m-0">
              Submission Progress
            </Title>
          </div>
        )
      }
    >
      {body}
    </Modal>
  );
}
