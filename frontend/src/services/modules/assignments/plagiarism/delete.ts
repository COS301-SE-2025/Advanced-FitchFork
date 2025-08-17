import type { ApiResponse } from "@/types/common";
import { api } from "@/utils/api";

export const deletePlagiarismCase = async (
  moduleId: number,
  assignmentId: number,
  caseId: number
): Promise<ApiResponse<null>> => {
  return api.delete(`/modules/${moduleId}/assignments/${assignmentId}/plagiarism/${caseId}`);
};

export const bulkDeletePlagiarismCases = async (
  moduleId: number,
  assignmentId: number,
  caseIds: number[]
): Promise<ApiResponse<null>> => {
  if (!caseIds || caseIds.length === 0) {
    throw new Error("case_ids cannot be empty");
  }

  const payload = { case_ids: caseIds };

  return api.delete(`/modules/${moduleId}/assignments/${assignmentId}/plagiarism/bulk`, { data: payload });
};
