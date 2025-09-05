import type {
  ApiResponse,
  PaginationRequest,
  PaginationResponse,
  Timestamp,
} from "@/types/common";
import { api, apiDownload } from "@/utils/api";

export interface GradeResponse extends Timestamp {
  id: number;
  assignment_id: number;
  user_id: number;
  submission_id?: number | null;
  score: number;
  username: string;
}

export interface ListGradesResponse extends PaginationResponse {
  grades: GradeResponse[];
}

/**
 * List grades for an assignment (with pagination, search, sort).
 */
export const listGrades = async (
  moduleId: number,
  assignmentId: number,
  params: PaginationRequest & { query?: string }
): Promise<ApiResponse<ListGradesResponse>> => {
  return api.get(`/modules/${moduleId}/assignments/${assignmentId}/grades`, params);
};

/**
 * Export grades for an assignment as CSV (browser download).
 */
export const exportGrades = async (
  moduleId: number,
  assignmentId: number
): Promise<void> => {
  return apiDownload(`/modules/${moduleId}/assignments/${assignmentId}/grades/export`);
};
