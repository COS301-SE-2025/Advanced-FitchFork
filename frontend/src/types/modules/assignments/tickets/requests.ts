import type { Ticket } from "./shared"

type TicketPayload = Omit<Ticket, "id" | "assignment_id" | "user_id" |"created_at" | "updated_at">

// ─────────────────────────────────────────────────────────────
// POST Request Types
// ─────────────────────────────────────────────────────────────

export type PostTicketRequest = Partial<TicketPayload>;