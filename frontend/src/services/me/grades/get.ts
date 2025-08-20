import type { ApiResponse, PaginationRequest, PaginationResponse } from "@/types/common"
import type { ModuleRole } from "@/types/modules";
import type { Grade } from "@/types/modules/grades";
import { api } from "@/utils/api";

// Query should allow searching by assignment title, username, and module code
// Sorting should work with score and created_at
type MyGradesOptions = {
  role?: ModuleRole;
  year?: number;
} & PaginationRequest;

type MyGradeItem = {
  user: {
    id: number;
    username: number;
  };
  module: {
    id: number;
    code: number;
  }
  assignment: {
    id: number;
    name: string;
    description: string;
  };
} & Grade;

type MyGradesResponse = ApiResponse<{grades: MyGradeItem[]} & PaginationResponse>;

export const getMyGrades = async (
  options: MyGradesOptions
): Promise<MyGradesResponse> => {
  return api.get("/me/grades", options)
}