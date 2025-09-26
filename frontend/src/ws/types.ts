// src/ws/types.ts

import type { SubmissionStatus } from "@/types/modules/assignments/submissions";
import type { User } from "@/types/users";

// ---------- Structured topics (mirror backend ClientTopic) ----------
export type ClientTopic =
  | { kind: 'system' }
  | { kind: 'system_admin' }
  | { kind: 'attendance_session'; session_id: number }
  | { kind: 'ticket_chat'; ticket_id: number }
  | { kind: 'assignment_submissions_staff'; assignment_id: number }
  | { kind: 'assignment_submissions_owner'; assignment_id: number; user_id: number };

// ---------- Frames we SEND ----------
export type WsIn =
  | { type: 'auth'; token: string }
  | { type: 'reauth'; token: string }
  | { type: 'subscribe'; topics: ClientTopic[]; since?: number }
  | { type: 'unsubscribe'; topics: ClientTopic[] }
  | { type: 'ping' }
  | { type: 'command'; name: string; topic?: ClientTopic; data: unknown };

// ---------- Frames we RECEIVE ----------
export type WsOutReady = { type: 'ready'; policy_version: number; exp: number | null };
export type WsOutPong = { type: 'pong' };
export type WsOutSubscribeOk = {
  type: 'subscribe_ok';
  accepted: string[];
  rejected: [string, string][];
};
export type WsOutUnsubscribeOk = { type: 'unsubscribe_ok'; topics: string[] };
export type WsOutError = {
  type: 'error';
  code: string;
  message: string;
  meta?: Record<string, string> | null;
};
export type WsOutEvent<T = unknown> = {
  type: 'event';
  event: string; // e.g. "submission.status"
  topic: string; // e.g. "assignment:42.submissions:user:7"
  v?: number | null; // optional version
  payload: T;
  ts: string; // RFC3339
};
export type WsOutAny<T = unknown> =
  | WsOutReady
  | WsOutPong
  | WsOutSubscribeOk
  | WsOutUnsubscribeOk
  | WsOutError
  | WsOutEvent<T>;

// ---------- Event payloads (mirror backend payload structs) ----------
export type SystemHealthGeneralPayload = {
  ts: string;
  load: { one: number; five: number; fifteen: number };
  code_manager: { running: number; waiting: number };
};

export type SystemHealthAdminPayload = {
  ts: string;
  env: string;
  host: string;
  uptime_seconds: number;
  load: { one: number; five: number; fifteen: number };
  cpu: { cores: number; avg_usage: number; per_core: number[] };
  memory: { total: number; used: number; swap_total: number; swap_used: number };
  disks: Array<{ name: string; total: number; available: number; file_system: string; mount_point: string }>;
  code_manager: { running: number; waiting: number; max_concurrent?: number | null };
};

export type MarkSummary = { earned: number; total: number };
export type SubmissionStatusPayload = {
  module_id: number;
  assignment_id: number;
  submission_id: number;
  username?: string | null;
  attempt: number;
  status: SubmissionStatus;
  mark?: MarkSummary | null;
  message?: string | null;
};

export type SubmissionNewPayload = {
  module_id: number;
  assignment_id: number;
  submission_id: number;
  username?: string | null;
  attempt: number;
  is_practice: boolean;
  created_at: string;
};

export type AttendanceSessionUpdated = {
  session_id: number;
  active: boolean;
  rotation_seconds: number;
  title: string;
  restrict_by_ip: boolean;
  allowed_ip_cidr?: string | null;
  created_from_ip?: string | null;
};
export type AttendanceMarked = {
  session_id: number;
  user_id: number;
  taken_at: string;
  count: number;
  method?: string | null;
};
export type AttendanceSessionDeleted = { session_id: number };

export type TicketMessage = {
  id: number;
  ticket_id: number;
  content: string;
  created_at: string;
  updated_at: string;
  user?: Pick<User, 'id' | 'username'> | null;
};
export type TicketMessageId = { id: number };

// ---------- Event name → payload type map ----------
export interface EventPayloadMap {
  'system.health': SystemHealthGeneralPayload;
  'system.health_admin': SystemHealthAdminPayload;

  'submission.status': SubmissionStatusPayload;
  'submission.new_submission': SubmissionNewPayload;

  'attendance.session_updated': AttendanceSessionUpdated;
  'attendance.marked': AttendanceMarked;
  'attendance.session_deleted': AttendanceSessionDeleted;

  'ticket.message_created': TicketMessage;
  'ticket.message_updated': TicketMessage;
  'ticket.message_deleted': TicketMessageId;
}

export type PayloadOf<Evt extends keyof EventPayloadMap> = EventPayloadMap[Evt];

// Minimal auth surface we’ll need from your AuthContext later
export interface AuthSurface {
  token: string | null;
  isExpired(): boolean;
  logout(): void;
}
