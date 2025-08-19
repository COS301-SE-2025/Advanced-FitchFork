import type { ApiResponse } from "@/types/common";
import { apiFetch } from "@/utils/api";

export const deleteTicket = (
  moduleId: number,
  assignmentId: number,
  ticketId: number
): Promise<ApiResponse<null>> => {
  return apiFetch(`/modules/${moduleId}/assignments/${assignmentId}/tickets/${ticketId}`, {
    method: "DELETE"
  })
}