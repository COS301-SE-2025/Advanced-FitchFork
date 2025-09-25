// src/ws/topics.ts
import type { ClientTopic } from './types';

// Build structured topics conveniently
export const Topics = {
  system(): ClientTopic { return { kind: 'system' }; },
  systemAdmin(): ClientTopic { return { kind: 'system_admin' }; },
  attendanceSession(session_id: number): ClientTopic { return { kind: 'attendance_session', session_id }; },
  ticketChat(ticket_id: number): ClientTopic { return { kind: 'ticket_chat', ticket_id }; },
  assignmentSubmissionsStaff(assignment_id: number): ClientTopic {
    return { kind: 'assignment_submissions_staff', assignment_id };
  },
  assignmentSubmissionsOwner(assignment_id: number, user_id: number): ClientTopic {
    return { kind: 'assignment_submissions_owner', assignment_id, user_id };
  },
} as const;

// Derive the exact server path string (must match backend `ClientTopic::path()`)
export function topicPath(t: ClientTopic): string {
  switch (t.kind) {
    case 'system': return 'system';
    case 'system_admin': return 'system:admin';
    case 'attendance_session': return `attendance:session:${t.session_id}`;
    case 'ticket_chat': return `tickets:${t.ticket_id}`;
    case 'assignment_submissions_staff': return `assignment:${t.assignment_id}.submissions:staff`;
    case 'assignment_submissions_owner': return `assignment:${t.assignment_id}.submissions:user:${t.user_id}`;
  }
}
