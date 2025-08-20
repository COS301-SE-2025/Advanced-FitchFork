import type { ApiResponse } from "@/types/common";
import type { PlagiarismCaseStatus } from "@/types/modules/assignments/plagiarism";
import { api } from "@/utils/api";


interface MinimalPlagiarismCaseUpdate {
  id: number;
  status: PlagiarismCaseStatus;
  updated_at: string; // ISO8601
}

export const flagPlagiarismCase = async (
  moduleId: number,
  assignmentId: number,
  caseId: number
): Promise<ApiResponse<MinimalPlagiarismCaseUpdate>> => {
  return api.patch(`/modules/${moduleId}/assignments/${assignmentId}/plagiarism/${caseId}/flag`);
};


export const reviewPlagiarismCase = async (
  moduleId: number,
  assignmentId: number,
  caseId: number
): Promise<ApiResponse<MinimalPlagiarismCaseUpdate>> => {
  return api.patch(`/modules/${moduleId}/assignments/${assignmentId}/plagiarism/${caseId}/review`);
};