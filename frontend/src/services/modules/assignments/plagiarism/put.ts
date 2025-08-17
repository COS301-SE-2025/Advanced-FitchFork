import type { ApiResponse } from "@/types/common";
import type { PlagiarismCaseStatus, PlagiarismCase } from "@/types/modules/assignments/plagiarism";
import { api } from "@/utils/api";

export const updatePlagiarismCase = async (
  moduleId: number,
  assignmentId: number,
  caseId: number,
  payload: {
    description?: string;
    status?: PlagiarismCaseStatus;
  }
): Promise<ApiResponse<PlagiarismCase>> => {
  return api.put(`/modules/${moduleId}/assignments/${assignmentId}/plagiarism/${caseId}`, payload);
};