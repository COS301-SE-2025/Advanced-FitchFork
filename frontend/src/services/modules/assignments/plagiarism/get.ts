import type { PaginationRequest } from "@/types/common";
import type { GetListPlagiarismCasesResponse, GetPlagiarismGraphResponse, PlagiarismCaseStatus } from "@/types/modules/assignments/plagiarism";
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
  }
): Promise<GetPlagiarismGraphResponse> => {
  return api.get(`/modules/${moduleId}/assignments/${assignmentId}/plagiarism/graph`, params);
};