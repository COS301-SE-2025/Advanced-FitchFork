import type {  PaginationRequest, PaginationResponse } from "@/types/common";
import type { ModuleRole } from "@/types/modules";
import type { Assignment, AssignmentStatus } from "@/types/modules/assignments";
import { api } from "@/utils/api";

// Query should allow search by assignment name and module code
// Sorting should work with due_date and available_from
type MyAssignmentsOptions = {
  role?: ModuleRole;
  year?: number;
  status?: AssignmentStatus;
} & PaginationRequest;

type MyAssignmentItem = {
  module: {
    id: number;
    code: string;
  }
} & Assignment;

type MyAssignmentsResponse = { assignments: MyAssignmentItem[] } & PaginationResponse;

export const getMyAssignments = async (
  options: MyAssignmentsOptions
) => {
  return api.get<MyAssignmentsResponse>("/me/assignments", options);
}