import type { PaginationRequest } from "@/types/common";
import type { GetTicketResponse, GetTicketsResponse, TicketStatus } from "@/types/modules/assignments/tickets"
import { apiFetch, buildQuery } from "@/utils/api";

export const listTickets = (
  moduleId: number,
  assignmentId: number,
  options: {
    status?: TicketStatus;
  } & PaginationRequest
): Promise<GetTicketsResponse> => {
  return apiFetch(`/modules/${moduleId}/assignments/${assignmentId}/tickets?${buildQuery(options)}`);
}

export const getTicket = (
  moduleId: number,
  assignmentId: number,
  ticketId: number
): Promise<GetTicketResponse> => {
  return apiFetch(`/modules/${moduleId}/assignments/${assignmentId}/tickets/${ticketId}`);
};