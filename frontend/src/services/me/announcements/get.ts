import type { ApiResponse, PaginationRequest, PaginationResponse } from "@/types/common"
import type { ModuleRole } from "@/types/modules";
import type { Announcement } from "@/types/modules/announcements";
import { api } from "@/utils/api";

// Query should allow searching by title, username, and module code
// Sorting should work with created_at and updated_at
type MyAnnouncementsOptions = {
  role?: ModuleRole;
  year?: number;
  pinned?: boolean;
} & PaginationRequest;

type MyAnnouncementItem = {
  user: {
    id: number;
    username: string;
  }
} & Announcement;

type MyAnnouncementsResponse = ApiResponse<{announcements: MyAnnouncementItem} & PaginationResponse>;

export const getMyAnnouncements = async (
  options: MyAnnouncementsOptions
): Promise<MyAnnouncementsResponse> => {
  return api.get("/me/announcements", options);
}