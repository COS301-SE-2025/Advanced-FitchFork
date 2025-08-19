import type { Timestamp } from "@/types/common";

export const TICKET_STATUSES = ['open', 'closed'] as const;
export type TicketStatus = (typeof TICKET_STATUSES)[number];

export interface Ticket extends Timestamp {
  id: number;
  assignment_id: number;
  user_id: number;
  title: string;
  description: string;
  status: TicketStatus;
}