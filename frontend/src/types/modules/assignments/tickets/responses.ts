import type { ApiResponse, PaginationResponse } from "@/types/common";
import type { Ticket } from "./shared";
import type { User } from "@/types/users";

// ─────────────────────────────────────────────────────────────
// GET Response Types
// ─────────────────────────────────────────────────────────────

export type GetTicketsResponse = ApiResponse<{
  tickets: Ticket[];
}& PaginationResponse>;

export type GetTicketResponse = ApiResponse<{
  ticket: Ticket;
  user: User;
}>;

// ─────────────────────────────────────────────────────────────
// POST Response Types
// ─────────────────────────────────────────────────────────────

export type PostTicketResponse = ApiResponse<Ticket>;