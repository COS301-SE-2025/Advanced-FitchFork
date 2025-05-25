import type { SortOption, Timestamp } from "./common";

// ─────────────────────────────────────────────────────────────
// ENUMS / SHARED TYPES
// ─────────────────────────────────────────────────────────────

/**
 * Allowed assignment types.
 */
export type AssignmentType = "Assignment" | "Practical";

/**
 * Payload shape shared between create/edit requests.
 */
export interface AssignmentPayload {
  name: string;
  description: string;
  assignment_type: AssignmentType;
  available_from: string;
  due_date: string;
}

// ─────────────────────────────────────────────────────────────
// CORE ENTITIES
// ─────────────────────────────────────────────────────────────

/**
 * Assignment entity including metadata and scheduling.
 */
export interface Assignment extends Timestamp {
  id: number;
  module_id: number;
  name: string;
  description: string;
  assignment_type: AssignmentType;
  available_from: string; // ISO
  due_date: string;
}

/**
 * Metadata for an uploaded assignment file.
 */
export interface AssignmentFile extends Timestamp {
  id: string;
  assignment_id: number;
  filename: string;
  path: string;
}

// ─────────────────────────────────────────────────────────────
// REQUEST TYPES
// ─────────────────────────────────────────────────────────────

export type CreateAssignmentRequest = AssignmentPayload;
export type EditAssignmentRequest = Partial<AssignmentPayload>;

export interface DeleteAssignmentFilesRequest {
  file_ids: string[];
}

export interface ListAssignmentsRequest {
  page: number;
  per_page: number;
  query?: string,
  sort?: SortOption[];
  name?: string;
  assignment_type?: AssignmentType;
  available_before?: string;
  available_after?: string;
  due_before?: string;
  due_after?: string;
}

// ─────────────────────────────────────────────────────────────
// RESPONSE TYPES
// ─────────────────────────────────────────────────────────────

export type CreateAssignmentResponse = Assignment;
export type EditAssignmentResponse = Assignment;
export type DeleteAssignmentResponse = null;

export interface AssignmentDetailsResponse extends Assignment {
  files: AssignmentFile[];
}

export interface ListAssignmentsResponse {
  assignments: Assignment[];
  page: number;
  per_page: number;
  total: number;
}

export type ListAssignmentFilesResponse = AssignmentFile[];
export type UploadAssignmentFilesResponse = AssignmentFile[];
export type DeleteAssignmentFilesResponse = null;

export interface PartialDeleteAssignmentFilesResponse {
  not_found: string[];
}
