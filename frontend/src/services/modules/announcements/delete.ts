import type { ApiResponse } from "@/types/common";
import { apiFetch } from "@/utils/api";

export const deleteAnnouncement = async (
  moduleId: number,
  announcementId: number
): Promise<ApiResponse<null>> => {
  return apiFetch(`/modules/${moduleId}/announcements/${announcementId}`, {
    method: "DELETE",
  });
};
