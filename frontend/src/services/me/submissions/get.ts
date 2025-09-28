import type { ApiResponse, PaginationRequest, PaginationResponse } from "@/types/common"
import type { ModuleRole } from "@/types/modules";
import type { Submission } from "@/types/modules/assignments/submissions";
import { api } from "@/utils/api"

// Query should allow search by module code, username and assignment name
// Sorting should work with score and created_at
type MySubmissionsOptions = {
  role?: ModuleRole;
  year?: number;
  is_late?: boolean;
  module_id?: number;
} & PaginationRequest;

type MySubmissionItem = {
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
    description: string;
  }
} & Submission;

type MySubmissionsResponse = ApiResponse<{submissions: MySubmissionItem[]} & PaginationResponse>;

export const getMySubmissions = async (
  options: MySubmissionsOptions
): Promise<MySubmissionsResponse> => {
  return api.get("/me/submissions", options)
}
