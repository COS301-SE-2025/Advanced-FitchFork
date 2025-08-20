import type { ApiResponse } from '@/types/common';
import type { TicketStatus } from '@/types/modules/assignments/tickets';
import { apiFetch } from '@/utils/api';

export const openTicket = (
  moduleId: number,
  assignmentId: number,
  ticketId: number
): Promise<ApiResponse<{ id: number; status: TicketStatus }>> => {
  return apiFetch(`/modules/${moduleId}/assignments/${assignmentId}/tickets/${ticketId}/open`, {
    method: 'PUT',
  });
};

export const closeTicket = (
  moduleId: number,
  assignmentId: number,
  ticketId: number
): Promise<ApiResponse<{ id: number; status: TicketStatus }>> => {
  return apiFetch(`/modules/${moduleId}/assignments/${assignmentId}/tickets/${ticketId}/close`, {
    method: 'PUT',
  });
};