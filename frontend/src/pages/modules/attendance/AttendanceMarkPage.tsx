import { useEffect, useMemo, useRef, useState } from 'react';
import { useNavigate, useSearchParams } from 'react-router-dom';
import { Button, Input, Alert, Tag, Space, Spin, Typography } from 'antd';
import { CheckCircleFilled, InfoCircleFilled, CloseCircleFilled } from '@ant-design/icons';
import { markAttendance } from '@/services/modules/attendance/post';
import { useAuth } from '@/context/AuthContext';
import { useUI } from '@/context/UIContext';

type AwaitState = { kind: 'idle' | 'loading' };
type DoneState =
  | { kind: 'success'; message: string }
  | { kind: 'already'; message: string }
  | { kind: 'error'; message: string };
type MarkState = AwaitState | DoneState;

const niceTime = (d = new Date()) =>
  d.toLocaleString([], {
    year: 'numeric',
    month: 'short',
    day: '2-digit',
    hour: '2-digit',
    minute: '2-digit',
    second: '2-digit',
  });

export default function AttendanceMarkPage() {
  const { isMobile } = useUI();
  const [sp] = useSearchParams();
  const navigate = useNavigate();
  const auth = useAuth();

  // Expect ?m=<module_id>&s=<session_id>&c=<code>
  const moduleId = Number(sp.get('m'));
  const sessionId = Number(sp.get('s'));
  const codeFromLink = sp.get('c')?.trim() ?? '';

  const hasModuleAndSession = useMemo(
    () => Number.isFinite(moduleId) && Number.isFinite(sessionId),
    [moduleId, sessionId],
  );
  const hasAll = useMemo(
    () => hasModuleAndSession && !!codeFromLink,
    [hasModuleAndSession, codeFromLink],
  );

  const [state, setState] = useState<MarkState>({ kind: 'idle' });
  const [manualOpen, setManualOpen] = useState<boolean>(false);
  const [manualCode, setManualCode] = useState<string>('');
  const triedAutoRef = useRef(false);

  // Fixed OTP length (backend now always returns 6-digit rolling code)
  const CODE_LENGTH = 6;

  // Auto-run if a code is in the URL (QR path).
  useEffect(() => {
    const run = async () => {
      if (!hasModuleAndSession) {
        setState({
          kind: 'error',
          message:
            'This link is incomplete. Ask your lecturer for the correct link or scan the QR in class.',
        });
        setManualOpen(false);
        return;
      }

      if (hasAll && !triedAutoRef.current) {
        triedAutoRef.current = true;

        if (!auth.user || auth.isExpired?.()) {
          setState({
            kind: 'error',
            message: 'You must be signed in to mark attendance.',
          });
          setManualOpen(true);
          return;
        }

        setState({ kind: 'loading' });
        const res = await markAttendance(moduleId, sessionId, codeFromLink, 'qr');

        if (res.success) {
          setState({
            kind: 'success',
            message: res.message || 'Attendance recorded successfully.',
          });
          setManualOpen(false);
        } else {
          const m = (res.message || '').toLowerCase();
          if (m.includes('already recorded') || m.includes('already marked')) {
            setState({
              kind: 'already',
              message: 'You’ve already been marked present for this session.',
            });
            setManualOpen(false);
          } else {
            setState({
              kind: 'error',
              message: res.message || 'Failed to record attendance.',
            });
            setManualOpen(true); // allow manual retry in a single, clear place
          }
        }
        return;
      }

      // No `?c=` → start idle with manual open
      setState({ kind: 'idle' });
      setManualOpen(true);
    };
    run();
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [hasAll, hasModuleAndSession, moduleId, sessionId, codeFromLink]);

  const handleManualSubmit = async (value?: string) => {
    const finalCode = (value ?? manualCode).trim();
    if (!hasModuleAndSession || !finalCode) return;
    if (!auth.user || auth.isExpired?.()) {
      setState({ kind: 'error', message: 'You must be signed in to mark attendance.' });
      return;
    }
    setState({ kind: 'loading' });
    const res = await markAttendance(moduleId, sessionId, finalCode, 'manual');

    if (res.success) {
      setState({ kind: 'success', message: res.message || 'Attendance recorded successfully.' });
      setManualCode('');
      setManualOpen(false);
    } else {
      const m = (res.message || '').toLowerCase();
      if (m.includes('already recorded') || m.includes('already marked')) {
        setState({
          kind: 'already',
          message: 'You’ve already been marked present for this session.',
        });
        setManualOpen(false);
      } else if (m.includes('invalid') || m.includes('expired') || m.includes('not active')) {
        setState({ kind: 'error', message: res.message || 'Invalid or expired code.' });
        setManualOpen(true);
      } else {
        setState({ kind: 'error', message: res.message || 'Failed to record attendance.' });
        setManualOpen(true);
      }
    }
  };

  const goModule = () =>
    Number.isFinite(moduleId)
      ? navigate(`/modules/${moduleId}/attendance`)
      : navigate('/dashboard');

  const tone =
    state.kind === 'success'
      ? { color: 'text-emerald-600', Icon: CheckCircleFilled as any }
      : state.kind === 'already'
        ? { color: 'text-blue-600', Icon: InfoCircleFilled as any }
        : state.kind === 'error'
          ? { color: 'text-rose-600', Icon: CloseCircleFilled as any }
          : { color: 'text-gray-500', Icon: InfoCircleFilled as any };

  return (
    <div className="min-h-screen w-full bg-gradient-to-br from-gray-50 to-gray-100 dark:from-gray-950 dark:to-gray-900">
      <main className="min-h-screen flex items-center justify-center px-4">
        <div className="w-full max-w-2xl">
          {/* Header */}
          <div className="mb-6 flex items-center justify-center gap-3 opacity-90">
            <img src="/ff_logo_favicon.svg" className="h-6 w-6" alt="FitchFork" />
            <span className="text-base font-medium text-gray-900 dark:text-gray-50">
              Attendance
            </span>
          </div>

          {/* Card */}
          <div className="rounded-2xl bg-white/70 dark:bg-gray-900/60 shadow-sm ring-1 ring-black/5 dark:ring-white/5 p-6 md:p-8">
            {/* Status / Title */}
            <div className="text-center">
              {state.kind === 'loading' ? (
                <div className="flex items-center justify-center py-3">
                  <Spin />
                </div>
              ) : (
                <div className="flex items-center justify-center gap-2">
                  {/* keep icon perfectly aligned with Title */}
                  <tone.Icon className={`${tone.color} text-2xl align-middle`} />
                  <Typography.Title
                    level={isMobile ? 4 : 3}
                    style={{ margin: 0, lineHeight: 1 }}
                    className="!m-0 !leading-none text-gray-900 dark:text-gray-50"
                  >
                    {state.kind === 'idle'
                      ? 'Mark your attendance'
                      : state.kind === 'success'
                        ? 'You’re marked present!'
                        : state.kind === 'already'
                          ? 'Already marked'
                          : 'Couldn’t mark attendance'}
                  </Typography.Title>
                </div>
              )}

              {(state.kind === 'success' || state.kind === 'already' || state.kind === 'error') && (
                <Typography.Paragraph className="mt-2 !mb-0 text-sm md:text-base text-gray-600 dark:text-gray-300">
                  {state.message}
                </Typography.Paragraph>
              )}
            </div>

            {/* Manual entry (hide when already marked or success) */}
            {hasModuleAndSession && state.kind !== 'success' && state.kind !== 'already' && (
              <div className="mt-6">
                {!manualOpen ? (
                  <div className="text-center">
                    <button
                      className="text-sm text-gray-700 dark:text-gray-200 underline underline-offset-4 hover:opacity-90"
                      onClick={() => setManualOpen(true)}
                    >
                      No camera? Enter code manually
                    </button>
                  </div>
                ) : (
                  <div
                    className="mx-auto max-w-md"
                    onKeyDown={(e) => {
                      if (e.key === 'Enter' && manualCode.length === CODE_LENGTH) {
                        void handleManualSubmit();
                      }
                    }}
                  >
                    <Alert
                      className="!mb-4"
                      type="info"
                      showIcon
                      message="Enter the 6-digit code shown by your lecturer."
                    />

                    {/* Center the OTP input blocks */}
                    <div className="flex justify-center">
                      <Input.OTP
                        length={CODE_LENGTH}
                        size="large"
                        variant="filled"
                        value={manualCode}
                        formatter={(str) => str.replace(/\D/g, '')}
                        onChange={(text) => {
                          const digitsOnly = (text ?? '').replace(/\D/g, '');
                          setManualCode(digitsOnly);
                          if (digitsOnly.length === CODE_LENGTH) {
                            void handleManualSubmit(digitsOnly);
                          }
                        }}
                        className="!w-full max-w-xs"
                        style={{ display: 'flex', justifyContent: 'center' }}
                      />
                    </div>

                    {/* Manual mode actions (centered) */}
                    <div className="mt-5 flex items-center justify-center">
                      <div className="flex items-center gap-2">
                        <Button type="text" onClick={() => setManualOpen(false)}>
                          Cancel
                        </Button>
                        <Button
                          type="primary"
                          onClick={() => handleManualSubmit()}
                          disabled={manualCode.length !== CODE_LENGTH || state.kind === 'loading'}
                        >
                          Mark attendance
                        </Button>
                      </div>
                    </div>
                  </div>
                )}
              </div>
            )}

            {/* Chips (no method) */}
            {(state.kind === 'success' || state.kind === 'already') && (
              <div className="mt-6">
                <Space wrap align="center" className="w-full justify-center">
                  <Tag>{niceTime()}</Tag>
                  {auth.user?.email && <Tag>{auth.user.email}</Tag>}
                </Space>
              </div>
            )}

            {/* Unified footer actions (hidden while manual input is open) */}
            {!manualOpen && (
              <div className="mt-8 flex items-center justify-center gap-2">
                {state.kind === 'error' && (
                  <>
                    <Button onClick={() => window.location.reload()}>Try again</Button>
                    <Button type="primary" onClick={goModule}>
                      Back to module
                    </Button>
                  </>
                )}

                {(state.kind === 'success' || state.kind === 'already') && (
                  <Button type="primary" onClick={goModule}>
                    Back to module
                  </Button>
                )}
              </div>
            )}
          </div>
        </div>
      </main>
    </div>
  );
}
