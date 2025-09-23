import { useEffect, useMemo, useRef, useState } from 'react';
import { WS_BASE_URL } from '@/config/api';
import { useAuth } from '@/context/AuthContext';

type GeneralPayload = {
  ts: string;
  load: { one: number; five: number; fifteen: number };
  code_manager: { running: number; waiting: number };
};

export function useSystemHealthWs() {
  const { token } = useAuth();
  const [data, setData] = useState<GeneralPayload | null>(null);
  const [connected, setConnected] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const wsRef = useRef<WebSocket | null>(null);
  const url = useMemo(() => {
    if (!token) return null;
    return `${WS_BASE_URL}/system/health?token=${encodeURIComponent(token)}`;
  }, [token]);

  const reconnectRef = useRef<number | undefined>(undefined);
  const backoffRef = useRef(0);
  const urlRef = useRef<string | null>(null);

  useEffect(() => {
    if (!url) {
      setConnected(false);
      try { wsRef.current?.close(); } catch {}
      wsRef.current = null;
      if (reconnectRef.current) { window.clearTimeout(reconnectRef.current); reconnectRef.current = undefined; }
      return;
    }

    if (urlRef.current === url && wsRef.current && (wsRef.current.readyState === WebSocket.OPEN || wsRef.current.readyState === WebSocket.CONNECTING)) {
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
            const msg = JSON.parse(ev.data);
            if (msg?.type === 'pong') return;
            setData(msg as GeneralPayload);
          } catch {}
        };
        ws.onclose = () => {
          setConnected(false);
          const n = Math.min(backoffRef.current++, 4);
          const delay = Math.floor((500 * Math.pow(2, n)) + Math.random() * 200);
          reconnectRef.current = window.setTimeout(connect, delay);
        };
        ws.onerror = () => { try { ws.close(); } catch {} };
      } catch (e: any) {
        const n = Math.min(backoffRef.current++, 4);
        const delay = Math.floor((500 * Math.pow(2, n)) + Math.random() * 200);
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

  return { data, connected, error };
}
