// src/ws/client.ts
import type {
  AuthSurface,
  ClientTopic,
  WsIn,
  WsOutAny,
  WsOutError,
  WsOutEvent,
  WsOutSubscribeOk,
  WsOutUnsubscribeOk,
} from './types';
import { SubscriptionRegistry, type EventName } from './registry';

export type SocketStatus = 'idle' | 'connecting' | 'open' | 'closing' | 'closed';

export interface SocketClientOptions {
  url: string;                 // e.g. `${WS_BASE_URL}/api/ws`
  auth: AuthSurface;           // from AuthContext
  registry: SubscriptionRegistry;
  log?: boolean;               // console logging
  pingIntervalMs?: number;     // default 20_000
  connectTimeoutMs?: number;   // default 10_000
  maxBackoffMs?: number;       // default 10_000
}

export class SocketClient {
  private ws: WebSocket | null = null;
  private status: SocketStatus = 'idle';
  private queue: WsIn[] = [];
  private backoff = 0;

  private reconnectTimer?: number;
  private pingTimer?: number;

  private readonly opts: Required<SocketClientOptions>;

  constructor(opts: SocketClientOptions) {
    this.opts = {
      pingIntervalMs: 20_000,
      connectTimeoutMs: 10_000,
      maxBackoffMs: 10_000,
      log: false,
      ...opts,
    } as Required<SocketClientOptions>;

    // Reconnect hints
    window.addEventListener('online', this.handleOnline);
    document.addEventListener('visibilitychange', this.handleVisibility);
  }

  // ------------- Public API -------------

  getStatus(): SocketStatus {
    return this.status;
  }

  ensureConnected(): void {
    if (this.status === 'open' || this.status === 'connecting') return;
    this.open();
  }

  /** Send frames (queued until OPEN). */
  send(frame: WsIn): void {
    if (this.status !== 'open' || !this.ws) {
      this.queue.push(frame);
      return;
    }
    try {
      this.ws.send(JSON.stringify(frame));
    } catch {
      // If send fails, re-queue and close to trigger reconnect
      this.queue.push(frame);
      this.safeClose('send_error');
    }
  }

  /** Structured subscribe (server expects ClientTopic[]). */
  subscribe(topics: ClientTopic[], since?: number): void {
    this.ensureConnected();
    this.send({ type: 'subscribe', topics, since });
  }

  unsubscribe(topics: ClientTopic[]): void {
    this.send({ type: 'unsubscribe', topics });
  }

  destroy(): void {
    window.removeEventListener('online', this.handleOnline);
    document.removeEventListener('visibilitychange', this.handleVisibility);
    this.stopPing();
    if (this.reconnectTimer) window.clearTimeout(this.reconnectTimer);
    this.status = 'closing';
    try { this.ws?.close(); } catch {}
    this.ws = null;
    this.status = 'closed';
  }

  // ------------- Internals -------------

  private open(): void {
    if (!this.opts.auth.token || this.opts.auth.isExpired()) {
      this.log('[ws] skip open: missing/expired token');
      return;
    }

    const url = new URL(this.opts.url);
    // backend auth is via HTTP guard; still pass token for completeness if your guard inspects it
    url.searchParams.set('token', this.opts.auth.token);

    this.log('[ws] connecting →', url.toString());
    this.status = 'connecting';
    const ws = new WebSocket(url.toString());
    this.ws = ws;

    const connectTimeout = window.setTimeout(() => {
      this.log('[ws] connect timeout');
      try { ws.close(); } catch {}
    }, this.opts.connectTimeoutMs);

    ws.onopen = () => {
      window.clearTimeout(connectTimeout);
      this.status = 'open';
      this.backoff = 0;
      this.log('[ws] open');

      // flush queue
      while (this.queue.length) {
        const f = this.queue.shift()!;
        try { ws.send(JSON.stringify(f)); } catch {}
      }
      // start keepalive
      this.startPing();
      this.send({ type: 'ping' });
    };

    ws.onmessage = (ev) => this.onMessage(ev.data);

    ws.onclose = () => {
      window.clearTimeout(connectTimeout);
      this.status = 'closed';
      this.log('[ws] close');
      this.stopPing();
      this.scheduleReconnect();
    };

    ws.onerror = () => {
      this.log('[ws] error');
      try { ws.close(); } catch {}
    };
  }

  private onMessage(raw: any): void {
    let msg: WsOutAny | null = null;
    try { msg = JSON.parse(String(raw)); } catch { return; }
    if (!msg || typeof msg !== 'object') return;

    switch ((msg as any).type) {
      case 'ready': {
        // { policy_version, exp }
        this.log('[ws] ready', msg);
        break;
      }
      case 'pong':
        // keepalive
        break;

      case 'subscribe_ok': {
        const m = msg as WsOutSubscribeOk;
        if (this.opts.log) {
          if (m.accepted.length) console.debug('[ws] subscribed:', m.accepted);
          if (m.rejected.length) console.warn('[ws] rejected:', m.rejected);
        }
        break;
      }

      case 'unsubscribe_ok': {
        const m = msg as WsOutUnsubscribeOk;
        if (this.opts.log) console.debug('[ws] unsubscribed:', m.topics);
        break;
      }

      case 'error': {
        const m = msg as WsOutError;
        console.error('[ws] error', m.code, m.message, m.meta);
        // If server hints auth misuse, force logout to be safe
        if (m.code === 'bad_request' && /auth/i.test(m.message)) {
          this.opts.auth.logout();
        }
        break;
      }

      case 'event': {
        const e = msg as WsOutEvent<unknown>;
        // event name should match keys in EventPayloadMap
        this.opts.registry.dispatch(e.event as EventName, e.topic, e.payload as any, e.ts, e.v ?? undefined);
        break;
      }
    }
  }

  private scheduleReconnect(): void {
    if (!navigator.onLine) {
      this.log('[ws] offline; waiting for online');
      return;
    }
    const n = Math.min(this.backoff++, 5);
    const delay = Math.min(
      this.opts.maxBackoffMs,
      Math.floor(500 * Math.pow(2, n) + Math.random() * 250)
    );
    if (this.reconnectTimer) window.clearTimeout(this.reconnectTimer);
    this.reconnectTimer = window.setTimeout(() => this.open(), delay);
    this.log('[ws] reconnect in', delay, 'ms');
  }

  private startPing(): void {
    this.stopPing();
    this.pingTimer = window.setInterval(() => this.send({ type: 'ping' }), this.opts.pingIntervalMs);
  }
  private stopPing(): void {
    if (this.pingTimer) {
      window.clearInterval(this.pingTimer);
      this.pingTimer = undefined;
    }
  }

  private handleOnline = () => {
    this.log('[ws] online');
    if (this.status === 'closed' || this.status === 'idle') this.open();
  };

  private handleVisibility = () => {
    if (document.visibilityState === 'visible' && this.status === 'open') {
      this.send({ type: 'ping' });
    }
  };

  private safeClose(why: string): void {
    if (this.ws && (this.status === 'open' || this.status === 'connecting')) {
      this.log('[ws] safeClose:', why);
      try { this.ws.close(); } catch {}
    }
  }

  private log(...args: any[]): void {
    if (this.opts.log) console.log(...args);
  }
}
