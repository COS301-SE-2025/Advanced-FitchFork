// src/types/modules/assignments/tickets/chat.ts
import type { TicketMessage as RestTicketMessage } from '@/types/modules/assignments/tickets/messages';
import type { TicketMessage as WsTicketMessage } from '@/ws/types';

export type ChatEntry = {
  id: number;
  content: string;
  createdAt: string; // ISO
  updatedAt: string; // ISO
  sender: string | null;
  user?: { id: number; username: string } | null;
  system?: boolean;
};

// REST -> UI
export function fromRestTicketMessage(m: RestTicketMessage): ChatEntry {
  return {
    id: m.id,
    content: m.content,
    createdAt: m.created_at,
    updatedAt: m.updated_at,
    user: m.user ? { id: m.user.id, username: m.user.username } : null,
    sender: m.user?.username ?? null,
  };
}

// WS -> UI
export function fromWsTicketMessage(m: WsTicketMessage): ChatEntry {
  return {
    id: m.id,
    content: m.content,
    createdAt: m.created_at,
    updatedAt: m.updated_at,
    user: m.user ? { id: m.user.id, username: m.user.username } : null,
    sender: m.user?.username ?? null,
  };
}
