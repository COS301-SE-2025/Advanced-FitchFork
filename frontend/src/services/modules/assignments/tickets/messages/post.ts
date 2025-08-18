import type { ApiResponse } from "@/types/common";
import type { TicketMessage } from "@/types/modules/assignments/tickets/messages";
import { api } from "@/utils/api";

/**
 * POST /modules/:moduleId/assignments/:assignmentId/tickets/:ticketId/messages
 * Create a new message in a ticket.
 */
export const createTicketMessage = async (
  moduleId: number,
  assignmentId: number,
  ticketId: number,
  content: string,
): Promise<ApiResponse<TicketMessage>> => {
  return api.post(
    `/modules/${moduleId}/assignments/${assignmentId}/tickets/${ticketId}/messages`,
    { content },
  );
};