// services/modules/assignments/plagiarism/delete.ts
import type { ApiResponse } from "@/types/common";
import { api } from "@/utils/api";

export const deleteMossReport = async (
  moduleId: number,
  assignmentId: number,
  reportId: number
): Promise<ApiResponse<null>> => {
  return api.delete(
    `/modules/${moduleId}/assignments/${assignmentId}/plagiarism/moss/reports/${reportId}`
  );
};

export const deletePlagiarismCase = async (
  moduleId: number,
  assignmentId: number,
  caseId: number
): Promise<ApiResponse<null>> => {
  return api.delete(
    `/modules/${moduleId}/assignments/${assignmentId}/plagiarism/${caseId}`
  );
};

export const bulkDeletePlagiarismCases = async (
  moduleId: number,
  assignmentId: number,
  caseIds: number[]
): Promise<ApiResponse<null>> => {
  if (!caseIds || caseIds.length === 0) {
    throw new Error("case_ids cannot be empty");
  }
  return api.delete(
    `/modules/${moduleId}/assignments/${assignmentId}/plagiarism/bulk`,
    { case_ids: caseIds }
  );
};
