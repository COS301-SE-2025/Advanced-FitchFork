import type { Timestamp, ApiResponse } from "@/types/common";

export interface SubmissionListUserInfo {
  id: number;
  email: string;
  student_number: string;
}

export interface SubmissionListItemBase extends Timestamp {
  id: number;
  attempt: number;
  filename: string;
}

// For staff (with user info)
export interface SubmissionListItemForStaff extends SubmissionListItemBase {
  user: SubmissionListUserInfo;
}

// For students (no user info)
export type SubmissionListItemForStudent = SubmissionListItemBase;

export interface PaginatedResponse<T> {
  items: T[];
  page: number;
  per_page: number;
  total: number;
}

export type GetSubmissionListResponse = ApiResponse<
  PaginatedResponse<SubmissionListItemForStaff | SubmissionListItemForStudent>
>;
