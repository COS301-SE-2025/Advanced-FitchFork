import type { ApiResponse } from "@/types/common";
import type { PlagiarismCase, MossFilterMode, HashScanData } from "@/types/modules/assignments/plagiarism";
import { api } from "@/utils/api";

export const createPlagiarismCase = async (
  moduleId: number,
  assignmentId: number,
  payload: {
    submission_id_1: number;
    submission_id_2: number;
    description: string;
    similarity: number;
  }
) => {
  return api.post<PlagiarismCase>(
    `/modules/${moduleId}/assignments/${assignmentId}/plagiarism`,
    payload
  );
};

// ---- MOSS run (async job w/ options) ----
export type RunMossPayload = {
  description: string;
  experimental?: boolean;
  max_matches?: number;
  show_limit?: number;
  filter_mode?: MossFilterMode;
  filter_patterns?: string[];
};

export type RunMossJobResponse = ApiResponse<{
  run_id: string;
  started_at: string; // RFC 3339
  message: string;
}>;

export const runMossCheck = async (
  moduleId: number,
  assignmentId: number,
  payload?: RunMossPayload
) => {
  return api.post<RunMossJobResponse>(
    `/modules/${moduleId}/assignments/${assignmentId}/plagiarism/moss`,
    payload ?? {}
  );
};

// Manual archive
export const archiveMossReport = async (
  moduleId: number,
  assignmentId: number
): Promise<ApiResponse<{}>> => {
  return api.post(
    `/modules/${moduleId}/assignments/${assignmentId}/plagiarism/moss/archive`
  );
};

export interface HashScanPayload {
  create_cases?: boolean;
}

export const runHashScan = async (
  moduleId: number,
  assignmentId: number,
  payload?: HashScanPayload
) => {
  return api.post<HashScanData>(
    `/modules/${moduleId}/assignments/${assignmentId}/plagiarism/hash-scan`,
    payload ?? {}
  );
};