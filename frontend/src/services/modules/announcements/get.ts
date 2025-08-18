import type { PaginationRequest, ApiResponse, PaginationResponse } from "@/types/common";
import type { Announcement } from "@/types/modules/announcements";
import { api } from "@/utils/api";

type GetListAnnouncementsResponse = ApiResponse<{
  announcements: Announcement[];
} & PaginationResponse>;

export const listAnnouncements = async (
  moduleId: number,
  params: {
    query?: string;
    pinned?: boolean;
  } & PaginationRequest
): Promise<GetListAnnouncementsResponse> => {
  return api.get(`/modules/${moduleId}/announcements`, params);
};

type GetAnnouncementResponse = ApiResponse<{
  announcement: Announcement;
  user: { id: number; username: string };
}>;

export const getAnnouncement = async (
  moduleId: number,
  announcementId: number
): Promise<GetAnnouncementResponse> => {
  return api.get(`/modules/${moduleId}/announcements/${announcementId}`);
};