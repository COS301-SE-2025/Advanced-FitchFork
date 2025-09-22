import { useEffect, useMemo, useRef, useState, useCallback } from 'react';
import { WS_BASE_URL } from '@/config/api';
import { useAuth } from '@/context/AuthContext';
import { getMaxConcurrent, setMaxConcurrent } from '@/services/system/code_manager';

export type AdminPayload = {
  ts: string;
  env: string;
  host: string;
  uptime_seconds: number;
  load: { one: number; five: number; fifteen: number };
  cpu: { cores: number; avg_usage: number; per_core?: number[] };
  memory: { total: number; used: number; swap_total: number; swap_used: number };
  disks: {
    name: string;
    total: number;
    available: number;
    file_system: string;
    mount_point?: string;
  }[];
  code_manager: { running: number; waiting: number; max_concurrent: number | null };
};

export function useSystemHealthAdminWs() {
  const { token, isAdmin } = useAuth();
  const [data, setData] = useState<AdminPayload | null>(null);
  const [connected, setConnected] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [maxConcurrent, setMaxConcurrentState] = useState<number | null>(null);
  const [saving, setSaving] = useState(false);
  const wsRef = useRef<WebSocket | null>(null);

  const url = useMemo(() => {
    if (!token || !isAdmin) return null;
    return `${WS_BASE_URL}/system/health/admin?token=${encodeURIComponent(token)}`;
  }, [token, isAdmin]);

  const reconnectRef = useRef<number | undefined>(undefined);
  const backoffRef = useRef(0);
  const urlRef = useRef<string | null>(null);

  useEffect(() => {
    if (!url) {
      setConnected(false);
      try { wsRef.current?.close(); } catch {}
      wsRef.current = null;
      if (reconnectRef.current) { window.clearTimeout(reconnectRef.current); reconnectRef.current = undefined; }
      if (!isAdmin) {
        setData(null);
        setMaxConcurrentState(null);
        setError(null);
      }
      return;
    }

    if (
      urlRef.current === url &&
      wsRef.current &&
      (wsRef.current.readyState === WebSocket.OPEN || wsRef.current.readyState === WebSocket.CONNECTING)
    ) {
      return;
    }

    urlRef.current = url;
    if (reconnectRef.current) { window.clearTimeout(reconnectRef.current); reconnectRef.current = undefined; }
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
          try {
            const raw = JSON.parse(ev.data) as any;
            if (raw?.type === 'pong') return;

            // Force uptime_seconds to be a number (required by type)
            const payload: AdminPayload = {
              ...raw,
              uptime_seconds: Number(raw?.uptime_seconds) || 0,
            };

            setData(payload);

            if (typeof payload?.code_manager?.max_concurrent === 'number') {
              setMaxConcurrentState(payload.code_manager.max_concurrent);
            }
          } catch {
            // ignore malformed messages
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
      if (reconnectRef.current) { window.clearTimeout(reconnectRef.current); reconnectRef.current = undefined; }
      try { wsRef.current?.close(); } catch {}
      wsRef.current = null;
    };
  }, [url]);

  const refreshMaxConcurrent = useCallback(async () => {
    if (!isAdmin) {
      return { success: false, data: null, message: 'Admin access required' } as const;
    }
    const res = await getMaxConcurrent();
    if (res.success && typeof res.data === 'number') {
      setMaxConcurrentState(res.data);
    }
    return res;
  }, [isAdmin]);

  const updateMaxConcurrent = useCallback(async (value: number) => {
    if (!isAdmin) {
      return { success: false, data: null, message: 'Admin access required' } as const;
    }
    setSaving(true);
    try {
      const res = await setMaxConcurrent(value);
      if (res.success) setMaxConcurrentState(value);
      return res;
    } finally {
      setSaving(false);
    }
  }, [isAdmin]);

  return { data, connected, error, maxConcurrent, saving, refreshMaxConcurrent, updateMaxConcurrent };
}
