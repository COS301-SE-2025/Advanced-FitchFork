import { useEffect, useMemo, useRef, useState } from 'react';
import { WS_BASE_URL } from '@/config/api';
import { useAuth } from '@/context/AuthContext';
import type { SubmissionStatus } from '@/types/modules/assignments/submissions';

type StatusMsgWire = {
  event: 'submission_status';
  submission_id: number;
  status: SubmissionStatus;     // 'queued' | 'running' | 'grading' | 'graded' | 'failed_*'
  ts?: string;                  // RFC3339
  message?: string;             // provided for failed_* statuses
  mark?: { earned: number; total: number }; 
};

// tolerate handshake/keepalive frames
type AnyMsg =
  | StatusMsgWire
  | { event: 'ready' }
  | { event: 'pong' }
  | { type: 'pong' }
  | Record<string, unknown>;

export type SubmissionProgress = {
  submissionId: number;
  assignmentId: number;
  userId: number;
  status?: SubmissionStatus;
  ts?: string;
  message?: string;
  mark?: { earned: number; total: number }; 
};

type ProgressIndex = Record<number, SubmissionProgress>;

type Options = { singleLatest?: boolean };

function joinUrl(base: string, path: string) {
  return `${base.replace(/\/+$/, '')}/${path.replace(/^\/+/, '')}`;
}

export function useSubmissionProgressWs(
  moduleId: number | null | undefined,
  assignmentId: number | null | undefined,
  userId?: number | null,
  options?: Options
) {
  const { token, user } = useAuth();
  const uid = userId ?? user?.id ?? null;

  const [connected, setConnected] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [lastEvent, setLastEvent] = useState<AnyMsg | null>(null);
  const [progressById, setProgressById] = useState<ProgressIndex>({});
  const wsRef = useRef<WebSocket | null>(null);

  const url = useMemo(() => {
    if (!token || !moduleId || !assignmentId || !uid) return null;

    // ws/modules/{module}/assignments/{assignment}/submissions/{user}
    const topic =
      `modules/${encodeURIComponent(String(moduleId))}` +
      `/assignments/${encodeURIComponent(String(assignmentId))}` +
      `/submissions/${encodeURIComponent(String(uid))}`;

    const base = joinUrl(WS_BASE_URL, topic);
    const q = new URLSearchParams({ token });
    return `${base}?${q.toString()}`;
  }, [token, moduleId, assignmentId, uid]);

  const reconnectRef = useRef<number | undefined>(undefined);
  const backoffRef = useRef(0);
  const urlRef = useRef<string | null>(null);

  useEffect(() => {
    if (!url) {
      setConnected(false);
      try { wsRef.current?.close(); } catch {}
      wsRef.current = null;
      if (reconnectRef.current) {
        window.clearTimeout(reconnectRef.current);
        reconnectRef.current = undefined;
      }
      return;
    }

    if (
      urlRef.current === url &&
      wsRef.current &&
      (wsRef.current.readyState === WebSocket.OPEN ||
        wsRef.current.readyState === WebSocket.CONNECTING)
    ) {
      return;
    }

    urlRef.current = url;
    if (reconnectRef.current) {
      window.clearTimeout(reconnectRef.current);
      reconnectRef.current = undefined;
    }
    backoffRef.current = 0;

    const connect = () => {
      if (!urlRef.current) return;
      try {
        const ws = new WebSocket(urlRef.current);
        wsRef.current = ws;

        ws.onopen = () => {
          setConnected(true);
          setError(null);
          backoffRef.current = 0;
          try { ws.send(JSON.stringify({ type: 'ping' })); } catch {}
        };

        ws.onmessage = (ev) => {
          let msg: AnyMsg | null = null;
          try { msg = JSON.parse(ev.data) as AnyMsg; } catch { /* ignore parse errors */ }
          if (!msg || typeof msg !== 'object') return;

          // ignore keepalives/handshake
          if ((msg as any).event === 'ready') return;
          if ((msg as any).event === 'pong' || (msg as any).type === 'pong') return;

          setLastEvent(msg);

          if ((msg as any).event === 'submission_status') {
            const s = msg as StatusMsgWire;
            if (!assignmentId || !uid) return;

            setProgressById((prev) => {
              const existing = prev[s.submission_id] ?? {
                submissionId: s.submission_id,
                assignmentId,
                userId: uid,
              };

              const updated: SubmissionProgress = {
                ...existing,
                status: s.status,
                ts: s.ts ?? existing.ts,
                message: s.message ?? existing.message,
                mark: s.mark ?? existing.mark,
              };

              return options?.singleLatest
                ? { [s.submission_id]: updated }
                : { ...prev, [s.submission_id]: updated };
            });
          }
        };

        ws.onclose = () => {
          setConnected(false);
          const n = Math.min(backoffRef.current++, 4);
          const delay = Math.floor(500 * Math.pow(2, n) + Math.random() * 200);
          reconnectRef.current = window.setTimeout(connect, delay);
        };

        ws.onerror = () => { try { ws.close(); } catch {} };
      } catch (e: any) {
        const n = Math.min(backoffRef.current++, 4);
        const delay = Math.floor(500 * Math.pow(2, n) + Math.random() * 200);
        reconnectRef.current = window.setTimeout(connect, delay);
        setError(e?.message || 'Failed to open WebSocket');
      }
    };

    connect();

    return () => {
      if (reconnectRef.current) {
        window.clearTimeout(reconnectRef.current);
        reconnectRef.current = undefined;
      }
      try { wsRef.current?.close(); } catch {}
      wsRef.current = null;
    };
  }, [url, options?.singleLatest, assignmentId, uid]);

  const latest = useMemo(() => {
    const items = Object.values(progressById);
    if (items.length === 0) return null;
    const sorted = items
      .slice()
      .sort(
        (a, b) =>
          (new Date(b.ts ?? 0).getTime() - new Date(a.ts ?? 0).getTime()) ||
          (b.submissionId - a.submissionId)
      );
    return sorted[0];
  }, [progressById]);

  return { connected, error, lastEvent, progressById, latest };
}
