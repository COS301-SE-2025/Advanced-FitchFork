import type { PaginationRequest } from "@/types/common";
import type {
  GetListPlagiarismCasesResponse,
  GetPlagiarismGraphResponse,
  MossReportListResponse,
  PlagiarismCaseStatus,
} from "@/types/modules/assignments/plagiarism";
import { api, apiDownload } from "@/utils/api";

// --- list reports (new source of truth) ---
export const listMossReports = async (
  moduleId: number,
  assignmentId: number
): Promise<MossReportListResponse> => {
  return api.get(
    `/modules/${moduleId}/assignments/${assignmentId}/plagiarism/moss/reports`
  );
};

export const listPlagiarismCases = async (
  moduleId: number,
  assignmentId: number,
  params: {
    status?: PlagiarismCaseStatus;
    report_id?: number;
  } & PaginationRequest
): Promise<GetListPlagiarismCasesResponse> => {
  return api.get(
    `/modules/${moduleId}/assignments/${assignmentId}/plagiarism`,
    params
  );
};

export const getPlagiarismGraph = async (
  moduleId: number,
  assignmentId: number,
  params?: {
    status?: PlagiarismCaseStatus;
    min_similarity?: number;
    max_similarity?: number;
    user?: string;
    report_id?: number;
  }
): Promise<GetPlagiarismGraphResponse> => {
  return api.get(
    `/modules/${moduleId}/assignments/${assignmentId}/plagiarism/graph`,
    params
  );
};

export const downloadMossArchiveByReport = async (
  moduleId: number,
  assignmentId: number,
  reportId: number
): Promise<void> => {
  return apiDownload(
    `/modules/${moduleId}/assignments/${assignmentId}/plagiarism/moss/reports/${reportId}/download`
  );
};
