import type { Timestamp } from "@/types/common";

export type TicketMessageUser = {
  id: number;
  username: string;
};

export interface TicketMessage extends Timestamp {
  id: number;
  ticket_id: number;
  content: string;
  user?: TicketMessageUser | null;
};