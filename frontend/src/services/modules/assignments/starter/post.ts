import type { ApiResponse } from "@/types/common";
import type { StarterPack } from "@/types/modules/assignments/starter";
import { api } from "@/utils/api";

/** GET /modules/:mid/assignments/:aid/starter → list available packs */
export function getStarterPacks(
  moduleId: number,
  assignmentId: number
): Promise<ApiResponse<StarterPack[]>> {
  return api.get<StarterPack[]>(
    `/modules/${moduleId}/assignments/${assignmentId}/starter`
  );
}
