import type { ApiResponse, PaginationRequest, PaginationResponse } from "@/types/common";
import type { TicketMessage } from "@/types/modules/assignments/tickets/messages";
import { api } from "@/utils/api";


type TicketMessageList = {
  tickets: TicketMessage[];
} & PaginationResponse;

/**
 * GET /modules/:moduleId/assignments/:assignmentId/tickets/:ticketId/messages
 * List ticket messages (paginated).
 */
export const listTicketMessages = async (
  moduleId: number,
  assignmentId: number,
  ticketId: number,
  params: PaginationRequest,
): Promise<ApiResponse<TicketMessageList>> => {
  return api.get(
    `/modules/${moduleId}/assignments/${assignmentId}/tickets/${ticketId}/messages`,
    { params },
  );
};