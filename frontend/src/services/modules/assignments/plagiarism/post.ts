import type { ApiResponse } from "@/types/common";
import type { PlagiarismCase } from "@/types/modules/assignments/plagiarism";
import { api } from "@/utils/api";

export const createPlagiarismCase = async (
  moduleId: number,
  assignmentId: number,
  payload: {
    submission_id_1: number;
    submission_id_2: number;
    description: string;
  }
): Promise<ApiResponse<PlagiarismCase>> => {
  return api.post(
    `/modules/${moduleId}/assignments/${assignmentId}/plagiarism`,
    payload
  );
};


interface RunMossPayload {
  language: string;
}

interface RunMossResult {
  report_url: string;   // URL returned by backend
}

export const runMossCheck = async (
  moduleId: number,
  assignmentId: number,
  payload: RunMossPayload
): Promise<ApiResponse<RunMossResult>> => {
  return api.post(`/modules/${moduleId}/assignments/${assignmentId}/plagiarism/moss`,payload);
};