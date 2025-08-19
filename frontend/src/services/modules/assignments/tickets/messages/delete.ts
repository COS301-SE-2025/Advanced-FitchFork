import type { ApiResponse } from "@/types/common";
import { api } from "@/utils/api";

/**
 * DELETE /modules/:moduleId/assignments/:assignmentId/tickets/:ticketId/messages/:messageId
 * Delete a message (author only).
 */
export const deleteTicketMessage = async (
  moduleId: number,
  assignmentId: number,
  ticketId: number,
  messageId: number,
): Promise<ApiResponse<{ id: number }>> => {
  return api.delete(
    `/modules/${moduleId}/assignments/${assignmentId}/tickets/${ticketId}/messages/${messageId}`,
  );
};