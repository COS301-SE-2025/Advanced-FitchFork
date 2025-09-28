import type { ApiResponse, PaginationRequest, PaginationResponse } from "@/types/common";
import type { ModuleRole } from "@/types/modules";
import { api } from "@/utils/api";

// Query should allow searching by assignment title, username, and module code
// Sorting should work with score and created_at
type MyGradesOptions = {
  role?: ModuleRole;
  year?: number;
  module_id?: number;
} & PaginationRequest;

export type MyGradeScore = {
  earned: number;
  total: number;
};

export type MyGradeItem = {
  id: number;
  score: MyGradeScore;
  percentage: number;
  created_at: string;
  updated_at: string;
  module: {
    id: number;
    code: string;
  };
  assignment: {
    id: number;
    title: string;
  };
  user: {
    id: number;
    username: string;
  };
};

export type MyGradesResponse = ApiResponse<
  { grades: MyGradeItem[] } & PaginationResponse
>;

export const getMyGrades = async (
  options: MyGradesOptions
): Promise<MyGradesResponse> => {
  return api.get("/me/grades", options);
};
