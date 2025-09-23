import type { PaginationRequest, PaginationResponse } from "@/types/common";
import type { ModuleRole } from "@/types/modules";
import type { Assignment, AssignmentStatus, AssignmentReadiness } from "@/types/modules/assignments";
import { api } from "@/utils/api";

// Query should allow search by assignment name and module code
// Sorting should work with due_date and available_from
type MyAssignmentsOptions = {
  role?: ModuleRole;
  module_id?: number;
  year?: number;
  status?: AssignmentStatus;
} & PaginationRequest;

export type AssignmentGrade = {
  percentage: number;
  earned: number;
  total: number;
};

export type AssignmentSubmissionSummary = {
  submitted: number;
  total_students: number;
};

export type MyAssignmentItem = {
  module: {
    id: number;
    code: string;
  };
  grade?: AssignmentGrade | null;
  submission_summary?: AssignmentSubmissionSummary | null;
  readiness?: AssignmentReadiness | null;
} & Assignment;

export type MyAssignmentsResponse = { assignments: MyAssignmentItem[] } & PaginationResponse;

export const getMyAssignments = async (
  options: MyAssignmentsOptions
) => {
  return api.get<MyAssignmentsResponse>("/me/assignments", options);
};
