// src/pages/modules/attendance/AttendanceSessionProjector.tsx
import { useEffect, useMemo, useState } from 'react';
import { useNavigate, useParams } from 'react-router-dom';
import { Button, QRCode, Space, Tooltip, Progress, message } from 'antd';
import {
  CloseOutlined,
  CopyOutlined,
  FullscreenExitOutlined,
  FullscreenOutlined,
  ReloadOutlined,
  PlayCircleOutlined,
  PauseCircleOutlined,
} from '@ant-design/icons';

import { useModule } from '@/context/ModuleContext';
import { useTheme } from '@/context/ThemeContext';
import { useAuth } from '@/context/AuthContext';

import type { AttendanceSession } from '@/types/modules/attendance';
import { getAttendanceSession, getCurrentAttendanceCode } from '@/services/modules/attendance/get';
import { editAttendanceSession } from '@/services/modules/attendance/put';
import { useAttendanceSessionWs } from '@/hooks/useAttendanceSessionWs';

export default function AttendanceSessionProjector() {
  const navigate = useNavigate();
  const { session_id } = useParams<{ session_id: string }>();
  const { id: moduleId } = useModule();
  const { isDarkMode } = useTheme();
  const auth = useAuth();

  const [loading, setLoading] = useState(true);
  const [session, setSession] = useState<AttendanceSession | null>(null);

  const [codeLoading, setCodeLoading] = useState(false);
  const [currentCode, setCurrentCode] = useState('');
  const [isFs, setIsFs] = useState(false);

  const qrIcon = isDarkMode ? '/ff_logo_dark.svg' : '/ff_logo_light.svg';

  // Load session
  const load = async () => {
    if (!session_id) return;
    setLoading(true);
    const res = await getAttendanceSession(moduleId, Number(session_id));
    setLoading(false);
    if (res.success) setSession(res.data);
    else message.error(res.message || 'Failed to load session');
  };
  useEffect(() => {
    load();
  }, [moduleId, session_id]);

  // Rotation ticker
  const [now, setNow] = useState<number>(() => Date.now());
  useEffect(() => {
    const t = window.setInterval(() => setNow(Date.now()), 1000);
    return () => window.clearInterval(t);
  }, []);
  const secondsLeft = useMemo(() => {
    if (!session) return undefined;
    const rot = Math.max(1, session.rotation_seconds);
    const sec = Math.floor(now / 1000);
    return rot - (sec % rot);
  }, [now, session]);

  // Fetch rotating code
  const fetchCode = async () => {
    if (!session) return;
    if (!session.active) {
      setCurrentCode('');
      return;
    }
    setCodeLoading(true);
    const res = await getCurrentAttendanceCode(moduleId, session.id);
    setCodeLoading(false);
    if (res.success) setCurrentCode(res.data);
    else message.error(res.message || 'Failed to fetch current code');
  };

  // Refresh at rotation boundaries
  useEffect(() => {
    if (!session?.active) return;
    let timerId: number | undefined;
    const schedule = () => {
      const rot = Math.max(1, session.rotation_seconds);
      const nowSec = Math.floor(Date.now() / 1000);
      const secsLeft = rot - (nowSec % rot);
      timerId = window.setTimeout(
        async () => {
          await fetchCode();
          schedule();
        },
        secsLeft * 1000 + 80,
      );
    };
    void fetchCode();
    schedule();
    return () => {
      if (timerId !== undefined) window.clearTimeout(timerId);
    };
  }, [session?.id, session?.rotation_seconds, session?.active]);

  // Build scan URL
  const scanUrl = useMemo(() => {
    if (!session || !currentCode) return '';
    const u = new URL(window.location.origin + '/attendance/mark');
    u.searchParams.set('m', String(moduleId));
    u.searchParams.set('s', String(session.id));
    u.searchParams.set('c', currentCode);
    return u.toString();
  }, [moduleId, session, currentCode]);

  // Fullscreen helpers
  const enterFs = async () => {
    try {
      await document.documentElement.requestFullscreen?.();
      setIsFs(true);
    } catch {}
  };
  const exitFs = async () => {
    try {
      await document.exitFullscreen?.();
      setIsFs(false);
    } catch {}
  };
  useEffect(() => {
    const onFsChange = () => setIsFs(Boolean(document.fullscreenElement));
    document.addEventListener('fullscreenchange', onFsChange);
    return () => document.removeEventListener('fullscreenchange', onFsChange);
  }, []);

  const closeAndBack = () => {
    if (document.fullscreenElement) document.exitFullscreen?.().catch(() => {});
    if (window.history.length > 1) navigate(-1);
    else navigate(`/modules/${moduleId}/attendance/sessions/${session_id}`);
  };

  // Toggle active
  const toggleActive = async (active: boolean) => {
    if (!session) return;
    const res = await editAttendanceSession(moduleId, session.id, { active });
    if (res.success) {
      setSession((prev) => (prev ? ({ ...prev, active } as AttendanceSession) : prev));
      if (active) {
        void fetchCode();
      } else {
        setCurrentCode('');
      }
    } else {
      message.error(res.message || 'Failed to update session');
    }
  };

  // WebSocket updates
  useAttendanceSessionWs({
    sessionId: session?.id,
    token: auth.token ?? null,
    onMarked: (p) => {
      setSession((prev) =>
        prev
          ? ({
              ...prev,
              attended_count: typeof p.count === 'number' ? p.count : prev.attended_count + 1,
            } as AttendanceSession)
          : prev,
      );
    },
    onSessionUpdated: (p) => {
      if (typeof p.active === 'boolean') {
        setSession((prev) => (prev ? ({ ...prev, active: p.active } as AttendanceSession) : prev));
        if (p.active) void fetchCode();
        else setCurrentCode('');
      }
    },
    onCodeRotated: () => void fetchCode(),
  });

  // Viewport & QR size
  const [vp, setVp] = useState({ w: window.innerWidth, h: window.innerHeight });
  useEffect(() => {
    const onResize = () => setVp({ w: window.innerWidth, h: window.innerHeight });
    window.addEventListener('resize', onResize);
    return () => window.removeEventListener('resize', onResize);
  }, []);
  const qrSize = Math.floor(Math.min(vp.w, vp.h) * 0.6);

  // Progress
  const rot = Math.max(1, session?.rotation_seconds ?? 30);
  const secsLeft = secondsLeft ?? rot;
  const secsElapsed = rot - secsLeft;
  const rotationPercent = Math.round((secsElapsed / rot) * 100);

  const attended = session?.attended_count ?? 0;
  const total = session?.student_count ?? 0;
  const attendancePct = total > 0 ? Math.round((attended / total) * 100) : 0;

  const topBarActions = (
    <Space>
      {session?.active ? (
        <Tooltip title="Disable session">
          <Button onClick={() => toggleActive(false)} icon={<PauseCircleOutlined />} danger>
            Disable
          </Button>
        </Tooltip>
      ) : (
        <Tooltip title="Activate session">
          <Button type="primary" onClick={() => toggleActive(true)} icon={<PlayCircleOutlined />}>
            Activate
          </Button>
        </Tooltip>
      )}

      <Tooltip title="Copy link">
        <Button
          icon={<CopyOutlined />}
          onClick={() => scanUrl && navigator.clipboard.writeText(scanUrl)}
          disabled={!session?.active || !scanUrl}
        />
      </Tooltip>

      <Tooltip title="Refresh now">
        <Button
          icon={<ReloadOutlined />}
          loading={codeLoading}
          onClick={fetchCode}
          disabled={!session?.active}
        />
      </Tooltip>

      {isFs ? (
        <Tooltip title="Exit Fullscreen">
          <Button icon={<FullscreenExitOutlined />} onClick={exitFs} />
        </Tooltip>
      ) : (
        <Tooltip title="Enter Fullscreen">
          <Button icon={<FullscreenOutlined />} onClick={enterFs} />
        </Tooltip>
      )}
      <Tooltip title="Close">
        <Button icon={<CloseOutlined />} onClick={closeAndBack} />
      </Tooltip>
    </Space>
  );

  return (
    <div className="fixed inset-0 z-50 flex flex-col bg-gray-50 text-gray-900 dark:bg-gray-950 dark:text-gray-50">
      <div className="flex items-center justify-between px-4 py-3 border-b border-gray-200 dark:border-gray-800">
        <div className="text-lg font-medium truncate">
          {session?.title ?? 'Attendance'}
          {loading && <span className="ml-2 opacity-60">· Loading…</span>}
        </div>
        {topBarActions}
      </div>

      {session?.active ? (
        <div className="flex-1 grid grid-cols-1 lg:grid-cols-2 place-items-center gap-6 p-6">
          <div className="flex items-center justify-center w-full h-full">
            <div className="p-3 rounded-2xl bg-white shadow-2xl">
              <QRCode
                value={scanUrl || ' '}
                errorLevel="H"
                size={qrSize}
                icon={qrIcon}
                iconSize={Math.floor(qrSize * 0.24)}
              />
            </div>
          </div>
          <div className="flex flex-col items-center justify-center gap-6 px-6 w-full">
            <div className="font-mono tracking-widest text-7xl sm:text-8xl md:text-9xl">
              {currentCode || (codeLoading ? '······' : '—')}
            </div>
            <div className="w-full max-w-xl">
              <Progress
                percent={rotationPercent}
                status="active"
                showInfo
                format={() => `${secsLeft}s left`}
              />
            </div>
            <div className="text-center space-y-1">
              <div className="text-lg sm:text-xl opacity-80">
                Rotates every <b>{session?.rotation_seconds ?? 30}s</b>
              </div>
            </div>
            <div className="flex flex-col items-center justify-center mt-4">
              <Progress type="circle" percent={attendancePct} width={180} />
              <div className="mt-3 text-lg font-medium">
                {attended} / {total} students
              </div>
            </div>
          </div>
        </div>
      ) : (
        <div className="flex-1 flex items-center justify-center p-8">
          <div className="max-w-3xl w-full text-center">
            <div className="relative flex items-center justify-center mb-6">
              <div className="absolute h-72 w-72 rounded-full ring-rose-500/40 blur-2xl animate-pulse" />
              <PauseCircleOutlined className="relative !text-rose-500 !text-[96px] drop-shadow-lg" />
            </div>
            <h1 className="text-3xl sm:text-4xl font-semibold tracking-tight">
              Session is disabled
            </h1>
            <p className="mt-3 text-base sm:text-lg opacity-80">
              Students can’t scan or mark attendance until you activate this session.
            </p>
            <div className="mt-6">
              <Button
                type="primary"
                size="large"
                icon={<PlayCircleOutlined />}
                onClick={() => toggleActive(true)}
                disabled={!session}
              >
                Activate session
              </Button>
            </div>
          </div>
        </div>
      )}
    </div>
  );
}
