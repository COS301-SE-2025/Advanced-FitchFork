import type { ApiResponse } from "@/types/common";
import type { TicketMessage } from "@/types/modules/assignments/tickets/messages";
import { api } from "@/utils/api";

/**
 * PUT /modules/:moduleId/assignments/:assignmentId/tickets/:ticketId/messages/:messageId
 * Edit an existing message (author only).
 */
export const editTicketMessage = async (
  moduleId: number,
  assignmentId: number,
  ticketId: number,
  messageId: number,
  content: string,
): Promise<ApiResponse<TicketMessage>> => {
  return api.put(
    `/modules/${moduleId}/assignments/${assignmentId}/tickets/${ticketId}/messages/${messageId}`,
    { content },
  );
};