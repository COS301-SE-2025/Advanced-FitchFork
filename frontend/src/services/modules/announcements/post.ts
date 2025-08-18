import type { ApiResponse } from "@/types/common";
import type { Announcement } from "@/types/modules/announcements";
import { api } from "@/utils/api";

export const createAnnouncement = async (
  moduleId: number,
  payload: {
    title: string;
    body: string;
    pinned: boolean;
  }
): Promise<ApiResponse<Announcement>> => {
  return api.post(`/modules/${moduleId}/announcements`, payload);
};
