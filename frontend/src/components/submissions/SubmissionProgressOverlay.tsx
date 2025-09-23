// src/components/submissions/SubmissionProgressOverlay.tsx
import { useEffect, useRef } from 'react';
import { useNavigate } from 'react-router-dom';
import { Modal, Result, Space, Typography, Button, Progress, Divider } from 'antd';
import {
  CheckCircleTwoTone,
  ClockCircleTwoTone,
  CloseCircleTwoTone,
  LoadingOutlined,
} from '@ant-design/icons';
import confetti, { type Options as ConfettiOptions } from 'canvas-confetti';

import { useSubmissionProgressWs } from '@/hooks/useSubmissionProgressWs';
import { scaleColor } from '@/utils/color';
import {
  SUBMISSION_STATUSES,
  type SubmissionStatus,
} from '@/types/modules/assignments/submissions';
import EventBus from '@/utils/EventBus';

const { Title } = Typography;

type Props = {
  moduleId: number;
  assignmentId: number;
  userId: number;
  onClose?: () => void;
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

const rand = (min: number, max: number) => Math.random() * (max - min) + min;

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

  // Live WS stream
  const { latest, connected } = useSubmissionProgressWs(moduleId, assignmentId, userId, {
    singleLatest: false,
  });

  // ---- Defer submit: fire on WS connect (once), or fallback after timeout ----
  const firedRef = useRef(false);
  useEffect(() => {
    if (!triggerSubmit || firedRef.current) return;

    let timer: number | undefined;

    // Fallback if WS doesn’t connect in time
    const timeoutMs = Math.max(500, wsConnectTimeoutMs ?? 2000);
    timer = window.setTimeout(() => {
      if (!firedRef.current) {
        firedRef.current = true;
        void triggerSubmit();
      }
    }, timeoutMs) as unknown as number;

    // Preferred: fire as soon as WS connects
    if (connected && !firedRef.current) {
      Promise.resolve().then(() => {
        if (!firedRef.current) {
          firedRef.current = true;
          void triggerSubmit();
        }
      });
    }

    return () => {
      if (timer) window.clearTimeout(timer);
    };
  }, [connected, triggerSubmit, wsConnectTimeoutMs]);

  // Active progress (WS only)
  const progress = latest ?? null;

  // Derived state
  const activeSubmissionId = submissionId ?? progress?.submissionId ?? null;

  const isSubmissionStatus = (v: unknown): v is SubmissionStatus =>
    typeof v === 'string' && (SUBMISSION_STATUSES as readonly string[]).includes(v);

  const rawStatus: SubmissionStatus | 'queued' | 'connecting' =
    progress?.status && isSubmissionStatus(progress.status)
      ? progress.status
      : connected
        ? 'queued'
        : 'connecting';

  const isFailed = typeof rawStatus === 'string' && rawStatus.startsWith('failed_');
  const isGraded = rawStatus === 'graded';
  const linearPercent =
    rawStatus === 'connecting'
      ? 5
      : isSubmissionStatus(rawStatus)
        ? STATUS_PROGRESS[rawStatus]
        : 10;

  const mark = progress?.mark as { earned: number; total: number } | undefined;
  const pct = mark && mark.total > 0 ? Math.round((mark.earned / mark.total) * 100) : null;
  const circleColor = pct != null ? scaleColor(pct, 'red-green') : undefined;

  const headerIcon = isFailed ? (
    <CloseCircleTwoTone twoToneColor="#ff4d4f" />
  ) : isGraded ? (
    // Icon won’t be shown when graded (title hidden), but keep for non-graded header.
    <CheckCircleTwoTone twoToneColor="#52c41a" />
  ) : (
    <ClockCircleTwoTone twoToneColor="#1677ff" />
  );

  const handleGoToSubmission = () => {
    if (!activeSubmissionId) return;
    onDone?.(activeSubmissionId);
    navigate(`/modules/${moduleId}/assignments/${assignmentId}/submissions/${activeSubmissionId}`);
    onClose?.();
  };

  // ===== Confetti + one-time EventBus emission on terminal state (WS-only) =====
  const prevStatusRef = useRef<SubmissionStatus | 'queued' | 'connecting' | null>(null);
  const firedForSubmissionRef = useRef<number | null>(null);
  const emittedTerminalRef = useRef<number | null>(null);

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
    const cameFromWs = !!progress; // all updates here are WS-driven
    const launch = launcherRef.current ?? confetti;

    const justGraded = prevStatusRef.current !== 'graded' && isGraded;
    const isPerfect = pct === 100 || (!!mark && mark.total > 0 && mark.earned === mark.total);
    const alreadyFiredForThis =
      firedForSubmissionRef.current != null &&
      activeSubmissionId != null &&
      firedForSubmissionRef.current === activeSubmissionId;

    const shouldFire =
      cameFromWs && justGraded && isPerfect && !alreadyFiredForThis && activeSubmissionId != null;

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

      const duration = 15_000;
      const end = Date.now() + duration;
      const defaults: ConfettiOptions = {
        startVelocity: 30,
        spread: 360,
        ticks: 60,
        zIndex: 2147483647,
        disableForReducedMotion: false,
      };
      const interval = window.setInterval(() => {
        const timeLeft = end - Date.now();
        if (timeLeft <= 0) return window.clearInterval(interval);
        const particleCount = 50 * (timeLeft / duration);
        launch({
          ...defaults,
          particleCount,
          origin: { x: rand(0.1, 0.3), y: Math.random() - 0.2 },
        });
        launch({
          ...defaults,
          particleCount,
          origin: { x: rand(0.7, 0.9), y: Math.random() - 0.2 },
        });
      }, 250);
    }

    // Emit EventBus once on terminal (WS-driven)
    const terminal = isGraded || isFailed;
    if (
      cameFromWs &&
      terminal &&
      activeSubmissionId &&
      emittedTerminalRef.current !== activeSubmissionId
    ) {
      emittedTerminalRef.current = activeSubmissionId;
      EventBus.emit('submission:updated', { id: activeSubmissionId });
    }

    prevStatusRef.current = rawStatus;
  }, [rawStatus, pct, isGraded, isFailed, activeSubmissionId, mark, progress]);

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
          {/* Big text BELOW the circle when graded */}
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
      destroyOnClose
      footer={
        <div style={{ display: 'flex', justifyContent: 'center', gap: 12 }}>
          <Button onClick={onClose}>Close</Button>
          <Button
            type="primary"
            onClick={handleGoToSubmission}
            disabled={!activeSubmissionId || (!isFailed && !isGraded)}
          >
            Go to submission
          </Button>
        </div>
      }
      // Hide modal header entirely when graded
      title={
        isGraded ? null : (
          <div className="flex items-center justify-center gap-2">
            {headerIcon}
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
