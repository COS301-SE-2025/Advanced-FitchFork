import type { ApiResponse } from "@/types/common";
import type { Announcement } from "@/types/modules/announcements";
import { apiFetch } from "@/utils/api";

export const updateAnnouncement = async (
  moduleId: number,
  announcementId: number,
  payload: {
    title: string;
    body: string;
    pinned: boolean;
  }
): Promise<ApiResponse<Announcement>> => {
  return apiFetch(`/modules/${moduleId}/announcements/${announcementId}`, {
    method: "PUT",
    body: JSON.stringify(payload),
  });
};
