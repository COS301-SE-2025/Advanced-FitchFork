// src/ws/hooks.tsx
import React, { createContext, useContext, useEffect, useMemo, useRef, useState } from 'react';
import type { AuthSurface, ClientTopic, EventPayloadMap } from './types';
import { SubscriptionRegistry, type EventHandler, type EventName } from './registry';
import { SocketClient, type SocketStatus } from './client';
import { topicPath } from './topics';

export interface WsContextValue {
  socket: SocketClient;
  registry: SubscriptionRegistry;
  status: SocketStatus;
}

const WsContext = createContext<WsContextValue | null>(null);

export function useWs(): WsContextValue {
  const ctx = useContext(WsContext);
  if (!ctx) throw new Error('useWs must be used within <WsProvider/>');
  return ctx;
}

export function WsProvider(props: {
  children: React.ReactNode;
  url: string; // e.g. `${WS_BASE_URL}/api/ws`
  auth: AuthSurface; // from your AuthContext
  log?: boolean;
}) {
  const { url, auth, log } = props;

  const [status, setStatus] = useState<SocketStatus>('idle');

  // IMPORTANT: give useRef an initial value (null) and widen the type
  const registryRef = useRef<SubscriptionRegistry | null>(null);
  const socketRef = useRef<SocketClient | null>(null);

  // Lazy init exactly once
  if (registryRef.current === null) {
    registryRef.current = new SubscriptionRegistry();
  }
  if (socketRef.current === null) {
    socketRef.current = new SocketClient({
      url,
      auth,
      registry: registryRef.current,
      log,
    });
  }

  // keep connection alive and reflect status
  useEffect(() => {
    const tick = window.setInterval(() => {
      setStatus(socketRef.current!.getStatus());
    }, 250);
    socketRef.current!.ensureConnected();
    return () => window.clearInterval(tick);
  }, []);

  const value = useMemo<WsContextValue>(
    () => ({
      socket: socketRef.current!, // safe after lazy init above
      registry: registryRef.current!, // safe after lazy init above
      status,
    }),
    [status],
  );

  return <WsContext.Provider value={value}>{props.children}</WsContext.Provider>;
}

// ---------------- Subscription hook ----------------

export type HandlerMap = {
  [K in keyof EventPayloadMap]?: EventHandler<K>;
};

export interface UseWsOptions {
  // optionally compute since per topic path; (not sent yet — wire when backend supports)
  sinceByTopic?: (topicPath: string) => number | undefined;
}

/**
 * Subscribes to the given topics and registers per-event handlers while the component is mounted.
 * - Dedupes subscribe/unsubscribe using a ref-counted registry.
 * - Strongly typed event handler map (event name → payload).
 */
export function useWsEvents(
  topics: ClientTopic[] | (() => ClientTopic[]),
  handlers: HandlerMap,
  _opts?: UseWsOptions,
) {
  const { socket, registry } = useWs();

  const handlersRef = useRef(handlers);
  handlersRef.current = handlers;

  // resolve topics once per change
  const topicList: ClientTopic[] = useMemo(
    () => (typeof topics === 'function' ? topics() : topics),
    // eslint-disable-next-line react-hooks/exhaustive-deps
    [typeof topics === 'function' ? undefined : JSON.stringify(topics)],
  );

  const topicPaths = useMemo(() => topicList.map(topicPath), [topicList]);
  const eventNames = useMemo(() => Object.keys(handlers) as EventName[], [handlers]);

  useEffect(() => {
    if (topicPaths.length === 0 || eventNames.length === 0) return;

    // Create distinct handler per event name so we know which was fired
    const perEventHandlers = new Map<EventName, EventHandler>();
    for (const ev of eventNames) {
      const h: EventHandler = (payload: any, meta) => {
        const fn = handlersRef.current[ev] as EventHandler | undefined;
        if (fn) fn(payload, meta);
      };
      perEventHandlers.set(ev, h);
    }

    // Register listeners and note which topics became first → need to subscribe
    const topicsBecameFirst = new Set<string>();
    for (const p of topicPaths) {
      for (const [ev, h] of perEventHandlers.entries()) {
        const becameFirst = registry.add(p, [ev], h);
        if (becameFirst) topicsBecameFirst.add(p);
      }
    }

    if (topicsBecameFirst.size > 0) {
      // Deduplicate structured topics for subscribe
      const toSub: ClientTopic[] = [];
      const seen = new Set<string>();
      for (const t of topicList) {
        const p = topicPath(t);
        if (topicsBecameFirst.has(p) && !seen.has(p)) {
          seen.add(p);
          toSub.push(t);
        }
      }
      if (toSub.length) socket.subscribe(toSub, undefined);
    }

    // Cleanup: remove handlers; collect topics that dropped to zero → unsubscribe
    return () => {
      const topicsDropped = new Set<string>();
      for (const p of topicPaths) {
        for (const [ev, h] of perEventHandlers.entries()) {
          const droppedToZero = registry.remove(p, [ev], h);
          if (droppedToZero) topicsDropped.add(p);
        }
      }

      if (topicsDropped.size > 0) {
        const toUnsub: ClientTopic[] = [];
        const seen = new Set<string>();
        for (const t of topicList) {
          const p = topicPath(t);
          if (topicsDropped.has(p) && !seen.has(p)) {
            seen.add(p);
            toUnsub.push(t);
          }
        }
        if (toUnsub.length) socket.unsubscribe(toUnsub);
      }
    };
  }, [registry, socket, JSON.stringify(topicPaths), JSON.stringify(eventNames)]);
}
