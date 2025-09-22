// src/hooks/useAttendanceSessionWs.ts
import { useEffect, useMemo, useRef, useState } from "react";
import { WS_BASE_URL } from "@/config/api";

type Envelope =
  | { type: "ping" }
  | { event: "pong"; payload?: unknown }
  | { event: "attendance_marked"; payload: { session_id: number; user_id: number; method: string; taken_at: string; count?: number } }
  | { event: "session_updated"; payload: { active?: boolean } }
  | { event: "code_rotated"; payload?: unknown }
  | Record<string, unknown>;

type Handlers = {
  onMarked?: (p: { session_id: number; user_id: number; method: string; taken_at: string; count?: number }) => void;
  onSessionUpdated?: (p: { active?: boolean }) => void;
  onCodeRotated?: () => void;
};

export function useAttendanceSessionWs(opts: {
  sessionId?: number | null;
  token?: string | null;
} & Handlers) {
  const { sessionId, token } = opts;

  // keep latest handlers in a ref so changes don't retrigger the WS effect
  const handlersRef = useRef<Handlers>({});
  handlersRef.current = {
    onMarked: opts.onMarked,
    onSessionUpdated: opts.onSessionUpdated,
    onCodeRotated: opts.onCodeRotated,
  };

  const [connected, setConnected] = useState(false);

  // Only build URL when we truly can connect; this also prevents flapping
  const url = useMemo(() => {
    if (!sessionId || !token) return null;
    const t = encodeURIComponent(token);
    return `${WS_BASE_URL}/attendance/sessions/${sessionId}?token=${t}`;
  }, [sessionId, token]);

  const wsRef = useRef<WebSocket | null>(null);
  const reconnectTimerRef = useRef<number | undefined>(undefined);
  const backoffRef = useRef(0); // attempt count
  const urlRef = useRef<string | null>(null);
  const closedByHookRef = useRef(false);

  // Visibility guard: pause connections when tab hidden
  const visible = typeof document !== "undefined"
    ? document.visibilityState === "visible"
    : true;

  useEffect(() => {
    if (!url || !visible) {
      // tear down if visible false or url null
      closedByHookRef.current = true;
      setConnected(false);
      if (reconnectTimerRef.current) {
        window.clearTimeout(reconnectTimerRef.current);
        reconnectTimerRef.current = undefined;
      }
      try { wsRef.current?.close(); } catch {}
      wsRef.current = null;
      return;
    }

    // avoid reconnect if URL unchanged and socket is still alive/connecting
    if (urlRef.current === url && wsRef.current &&
       (wsRef.current.readyState === WebSocket.OPEN || wsRef.current.readyState === WebSocket.CONNECTING)) {
      return;
    }

    // new URL → reset state & connect
    urlRef.current = url;
    closedByHookRef.current = false;
    if (reconnectTimerRef.current) {
      window.clearTimeout(reconnectTimerRef.current);
      reconnectTimerRef.current = undefined;
    }
    backoffRef.current = 0;

    const connect = () => {
      if (!urlRef.current) return;
      try {
        const ws = new WebSocket(urlRef.current);
        wsRef.current = ws;

        ws.onopen = () => {
          setConnected(true);
          backoffRef.current = 0;
          // optional keepalive ping
          try { ws.send(JSON.stringify({ type: "ping" })); } catch {}
        };

        ws.onmessage = (ev) => {
          let data: Envelope | null = null;
          try { data = JSON.parse(ev.data); } catch { return; }
          if (!data) return;

          // framework ping/pong
          if ((data as any).type === "ping") {
            try { ws.send(JSON.stringify({ type: "pong" })); } catch {}
            return;
          }

          const event = (data as any).event;
          const payload = (data as any).payload;

          switch (event) {
            case "attendance_marked":
              handlersRef.current.onMarked?.(payload);
              break;
            case "session_updated":
              handlersRef.current.onSessionUpdated?.(payload);
              break;
            case "code_rotated":
              handlersRef.current.onCodeRotated?.();
              break;
            case "pong":
            default:
              break;
          }
        };

        ws.onclose = () => {
          setConnected(false);
          if (closedByHookRef.current) return;
          // backoff with jitter: 500ms * 2^n (cap ~8s)
          const n = Math.min(backoffRef.current++, 4);
          const delay = Math.floor((500 * Math.pow(2, n)) + Math.random() * 200);
          reconnectTimerRef.current = window.setTimeout(connect, delay);
        };

        ws.onerror = () => {
          try { ws.close(); } catch {}
        };
      } catch {
        // schedule a retry if constructor throws
        const n = Math.min(backoffRef.current++, 4);
        const delay = Math.floor((500 * Math.pow(2, n)) + Math.random() * 200);
        reconnectTimerRef.current = window.setTimeout(connect, delay);
      }
    };

    connect();

    return () => {
      // teardown for URL/visibility/unmount changes
      closedByHookRef.current = true;
      if (reconnectTimerRef.current) {
        window.clearTimeout(reconnectTimerRef.current);
        reconnectTimerRef.current = undefined;
      }
      setConnected(false);
      try { wsRef.current?.close(); } catch {}
      wsRef.current = null;
    };
  }, [url, visible]);

  // visibility listener (so `visible` changes without remount)
  useEffect(() => {
    const onVis = () => {
      // trigger effect by changing `visible` via state? Simpler:
      // do nothing here; the dep on `visible` above is already from document.visibilityState
      // but we need to force a render when it changes:
      setConnected((v) => v); // noop set to bump render
    };
    document.addEventListener("visibilitychange", onVis);
    return () => document.removeEventListener("visibilitychange", onVis);
  }, []);

  return { connected };
}
