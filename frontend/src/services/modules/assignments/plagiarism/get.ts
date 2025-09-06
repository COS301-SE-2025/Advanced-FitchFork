import type { ApiResponse, PaginationRequest } from "@/types/common";
import type { GetListPlagiarismCasesResponse, GetMossReportResponse, GetPlagiarismGraphResponse, PlagiarismCaseStatus } from "@/types/modules/assignments/plagiarism";
import { api } from "@/utils/api";


export const listPlagiarismCases = async (
  moduleId: number,
  assignmentId: number,
  params: {
    status?: PlagiarismCaseStatus;
  } & PaginationRequest

): Promise<GetListPlagiarismCasesResponse> => {
  return api.get(`/modules/${moduleId}/assignments/${assignmentId}/plagiarism`, params);
};

export const getPlagiarismGraph = async (
  moduleId: number,
  assignmentId: number,
  params?: {
    status?: PlagiarismCaseStatus;
    min_similarity?: number;
    max_similarity?: number;
    user?: string;
  }
): Promise<GetPlagiarismGraphResponse> => {
  return api.get(`/modules/${moduleId}/assignments/${assignmentId}/plagiarism/graph`, params);
};

export const getMossReport = async (
  moduleId: number,
  assignmentId: number,
): Promise<ApiResponse<GetMossReportResponse>> => {
  return api.get(`/modules/${moduleId}/assignments/${assignmentId}/plagiarism/moss`);
}