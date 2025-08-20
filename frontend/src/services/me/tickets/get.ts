import type { ApiResponse, PaginationRequest, PaginationResponse } from "@/types/common"
import type { ModuleRole } from "@/types/modules";
import type { Ticket, TicketStatus } from "@/types/modules/assignments/tickets";
import { api } from "@/utils/api";

// Query should allow searching by Ticket title, username, module code and assignment name
// Sorting should work with created_at
type MyTicketsOptions = {
  role?: ModuleRole;
  year?: number;
  status?: TicketStatus
} & PaginationRequest;

type MyTicketItem = {
  user: {
    id: number;
    username: string;
  };
  module: {
    id: number;
    code: string;
  };
  assignment: {
    id: number;
    name: string;
  }
} & Ticket;

type MyTicketsResponse = ApiResponse<{tickets: MyTicketItem} & PaginationResponse>;

export const getMyTickets = async (
  options: MyTicketsOptions
): Promise<MyTicketsResponse> => {
  return api.get("/me/tickets", options)
}