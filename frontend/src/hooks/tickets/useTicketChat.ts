import { useEffect, useRef, useState, useCallback } from 'react';
import { message as toast } from '@/utils/message';
import {
  listTicketMessages,
  createTicketMessage,
  editTicketMessage,
  deleteTicketMessage,
} from '@/services/modules/assignments/tickets/messages';
import type { TicketMessage } from '@/types/modules/assignments/tickets/messages';
import { WS_BASE_URL } from '@/config/api';

export type ChatEntry = {
  id: number;
  userId: number | null;
  sender: string;
  content: string;
  createdAt: string; // ISO
  updatedAt: string; // ISO
  system?: boolean;
};

type WsEnvelope<T extends string, P> = {
  event: T;
  topic: string;
  payload: P;
  ts: string;
};

type WsEvent =
  | WsEnvelope<'message_created', TicketMessage>
  | WsEnvelope<'message_updated', TicketMessage>
  | WsEnvelope<'message_deleted', { id: number }>
  | WsEnvelope<'typing', { sender: string }>;

const toEntry = (m: TicketMessage): ChatEntry => ({
  id: m.id,
  userId: m.user?.id ?? null,                         // <-- NEW
  sender: m.user?.username ?? 'Unknown',
  content: m.content,
  createdAt: m.created_at ?? new Date().toISOString(),
  updatedAt: m.updated_at ?? m.created_at ?? new Date().toISOString(),
});

export function useTicketChat(opts: {
  moduleId?: number | null;
  assignmentId?: number | null;
  ticketId?: number | null;
  token?: string | null;
  username?: string | null; // used for typing
}) {
  const { moduleId, assignmentId, ticketId, token, username } = opts;

  // WS can run with just ticketId+token now
  const canUseWs = !!ticketId && !!token;
  // REST still needs module+assignment+ticket
  const canUseRest = !!moduleId && !!assignmentId && !!ticketId;

  const [messages, setMessages] = useState<ChatEntry[]>([]);
  const [typingMap, setTypingMap] = useState<Record<string, number>>({});
  const [loaded, setLoaded] = useState(false);

  const wsRef = useRef<WebSocket | null>(null);

  // prune expired typing entries every second
  useEffect(() => {
    const id = window.setInterval(() => {
      const now = Date.now();
      setTypingMap((prev) => {
        const next: Record<string, number> = {};
        let changed = false;
        for (const [name, t] of Object.entries(prev)) {
          if (t > now) next[name] = t;
          else changed = true;
        }
        return changed ? next : prev;
      });
    }, 1000);
    return () => clearInterval(id);
  }, []);

  // Reset state when target chat changes
  useEffect(() => {
    setMessages([]);
    setTypingMap({});
    // if we can't fetch, consider "loaded" to avoid premature empty UI
    setLoaded(!canUseRest);
  }, [moduleId, assignmentId, ticketId, token, canUseRest]);

  // Initial load via REST (unchanged)
  useEffect(() => {
    if (!canUseRest) return;

    let cancelled = false;
    (async () => {
      try {
        const res = await listTicketMessages(moduleId!, assignmentId!, ticketId!, {
          page: 1,
          per_page: 200,
        });
        if (!res.success) {
          if (!cancelled) toast.error(res.message || 'Failed to load messages');
          return;
        }
        const entries = res.data.tickets
          .map(toEntry)
          .sort((a, b) => new Date(a.createdAt).getTime() - new Date(b.createdAt).getTime());
        if (!cancelled) setMessages(entries);
      } catch {
        if (!cancelled) toast.error('Failed to load messages');
      } finally {
        if (!cancelled) setLoaded(true);
      }
    })();

    return () => {
      cancelled = true;
    };
  }, [canUseRest, moduleId, assignmentId, ticketId]);

  useEffect(() => {
    if (!canUseWs) return;

    const url = `${WS_BASE_URL}/tickets/${ticketId}?token=${encodeURIComponent(token!)}`;
    const ws = new WebSocket(url);
    wsRef.current = ws;

    // WS handler
    ws.onmessage = (ev) => {
      try {
        const data: WsEvent = JSON.parse(ev.data);
        switch (data.event) {
          case 'message_created': {
            const entry = toEntry(data.payload); // includes userId + sender
            setMessages((prev) => (prev.some((m) => m.id === entry.id) ? prev : [...prev, entry]));
            break;
          }
          case 'message_updated': {
            const p = data.payload;
            const updatedAt = p.updated_at || p.created_at || new Date().toISOString();
            setMessages((prev) =>
              prev.map((m) => {
                if (m.id !== p.id) return m;
                // Do NOT clobber author if server sent user: null
                const keepAuthor = !('user' in p) || p.user == null;
                return {
                  ...m,
                  content: p.content,
                  updatedAt,
                  ...(keepAuthor
                    ? {}
                    : {
                        userId: p.user?.id ?? m.userId,
                        sender: p.user?.username ?? m.sender,
                      }),
                };
              }),
            );
            break;
          }
          case 'message_deleted': {
            setMessages((prev) => prev.filter((m) => m.id !== data.payload.id));
            break;
          }
          case 'typing': {
            const sender = data.payload.sender;
            if (sender && sender !== username) {
              setTypingMap((prev) => ({ ...prev, [sender]: Date.now() + 3000 }));
            }
            break;
          }
        }
      } catch {
        // ignore malformed payloads
      }
    };

    return () => {
      ws.close();
      wsRef.current = null;
    };
  }, [canUseWs, ticketId, token, username]);

  // send typing
  const emitTyping = useCallback(() => {
    const ws = wsRef.current;
    if (!ws || ws.readyState !== WebSocket.OPEN) return;
    try {
      ws.send(JSON.stringify({ type: 'typing', sender: username || 'anonymous' }));
    } catch {}
  }, [username]);

  const send = useCallback(
    async (content: string) => {
      if (!canUseRest) return;
      try {
        const res = await createTicketMessage(moduleId!, assignmentId!, ticketId!, content);
        if (!res.success) toast.error(res.message || 'Failed to send message');
      } catch {
        toast.error('Failed to send message');
      }
    },
    [canUseRest, moduleId, assignmentId, ticketId],
  );

  const update = useCallback(
    async (id: number, content: string) => {
      if (!canUseRest) return;
      try {
        const res = await editTicketMessage(moduleId!, assignmentId!, ticketId!, id, content);
        if (!res.success) toast.error(res.message || 'Failed to update message');
      } catch {
        toast.error('Failed to update message');
      }
    },
    [canUseRest, moduleId, assignmentId, ticketId],
  );

  const remove = useCallback(
    async (id: number) => {
      if (!canUseRest) return;
      try {
        const res = await deleteTicketMessage(moduleId!, assignmentId!, ticketId!, id);
        if (!res.success) toast.error(res.message || 'Failed to delete message');
      } catch {
        toast.error('Failed to delete message');
      }
    },
    [canUseRest, moduleId, assignmentId, ticketId],
  );

  const typingUsers = Object.keys(typingMap).filter((n) => n && n !== username);
  const typingText =
    typingUsers.length === 0
      ? ''
      : typingUsers.length === 1
      ? `${typingUsers[0]} is typing…`
      : typingUsers.length === 2
      ? `${typingUsers[0]} and ${typingUsers[1]} are typing…`
      : `${typingUsers[0]}, ${typingUsers[1]} and ${typingUsers.length - 2} others are typing…`;

  return { canUse: canUseRest, messages, send, update, remove, emitTyping, typingText, loaded };
}
