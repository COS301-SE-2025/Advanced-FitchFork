import type { ApiResponse } from "@/types/common";
import type { ModuleRole } from "@/types/modules";
import type { ActivityFeed } from "@/types/me/activity";
import { api } from "@/utils/api";

export type GetMyActivityParams = {
  page?: number;
  per_page?: number;
  module_id?: number;
  role?: ModuleRole;
  types?: string;
};

export type GetMyActivityResponse = ApiResponse<ActivityFeed>;

export const getMyActivity = async (
  params: GetMyActivityParams = {}
): Promise<GetMyActivityResponse> => {
  return api.get<ActivityFeed>("/me/activity", params);
};
