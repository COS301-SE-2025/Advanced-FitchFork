import type { PostTicketRequest, PostTicketResponse } from "@/types/modules/assignments/tickets";
import { apiFetch } from "@/utils/api";

export const createTicket = async (
  moduleId: number, 
  assignmentId: number,
  payload: PostTicketRequest
): Promise<PostTicketResponse> => {
  return apiFetch(`/modules/${moduleId}/assignments/${assignmentId}/tickets`, {
    method: "POST",
    body: JSON.stringify(payload),
  })
}