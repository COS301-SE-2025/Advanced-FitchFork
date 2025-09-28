import type { ApiResponse, PaginationRequest, PaginationResponse } from "@/types/common";
import type { ModuleRole } from "@/types/modules";
import { api } from "@/utils/api";

export type PlagiarismSubmission = {
  submission_id: number;
  user_id: number;
  username: string;
};

export type PlagiarismCaseItem = {
  id: number;
  assignment: {
    id: number;
    name: string;
  };
  module: {
    id: number;
    code: string;
  };
  status: string;
  similarity: number;
  lines_matched: number;
  created_at: string;
  updated_at: string;
  submission_1: PlagiarismSubmission;
  submission_2: PlagiarismSubmission;
};

export type GetMyPlagiarismCasesResponse = ApiResponse<
  { cases: PlagiarismCaseItem[] } & PaginationResponse
>;

export type GetMyPlagiarismCasesParams = {
  module_id?: number;
  assignment_id?: number;
  status?: string;
  sort?: string;
  role?: ModuleRole;
} & PaginationRequest;

export const getMyPlagiarismCases = async (
  params: GetMyPlagiarismCasesParams,
): Promise<GetMyPlagiarismCasesResponse> => {
  return api.get("/me/plagiarism", params);
};
