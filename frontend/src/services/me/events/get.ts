import type { ApiResponse } from "@/types/common";
import type { ModuleRole } from "@/types/modules";
import { api } from "@/utils/api";

export type GetMyEventsParams = {
  from?: string;
  to?: string;
  role?: ModuleRole;
  module_id?: number;
};

export type CalendarEvent = {
  type: string;
  content: string;
  module_id: number;
  assignment_id: number;
};

export type GetMyEventsData = {
  events: Record<string, CalendarEvent[]>;
};

export type GetMyEventsResponse = ApiResponse<GetMyEventsData>;

export const getMyEvents = async (
  params: GetMyEventsParams = {}
): Promise<GetMyEventsResponse> => {
  return api.get<GetMyEventsData>("/me/events", params);
};
