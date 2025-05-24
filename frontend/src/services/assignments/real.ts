import type {
  AssignmentDetailsResponse,
  CreateAssignmentRequest,
  CreateAssignmentResponse,
  DeleteAssignmentFilesRequest,
  DeleteAssignmentFilesResponse,
  DeleteAssignmentResponse,
  EditAssignmentRequest,
  EditAssignmentResponse,
  ListAssignmentsRequest,
  ListAssignmentsResponse,
  PartialDeleteAssignmentFilesResponse,
  UploadAssignmentFilesResponse,
  ListAssignmentFilesResponse,
} from "@/types/assignments";
import { apiFetch, type ApiResponse } from "@/utils/api";

/**
 * AssignmentsService handles API requests related to module assignments and their attachments.
 */
export const AssignmentsService = {
  /**
   * Create a new assignment under a specific module.
   */
  createAssignment: (
    moduleId: string,
    payload: CreateAssignmentRequest
  ): Promise<ApiResponse<CreateAssignmentResponse>> => {
    return apiFetch(`/modules/${moduleId}/assignments`, {
      method: "POST",
      body: JSON.stringify(payload),
    });
  },

  /**
   * Edit an existing assignment by ID.
   */
  editAssignment: (
    moduleId: string,
    assignmentId: string,
    payload: EditAssignmentRequest
  ): Promise<ApiResponse<EditAssignmentResponse>> => {
    return apiFetch(`/modules/${moduleId}/assignments/${assignmentId}`, {
      method: "PUT",
      body: JSON.stringify(payload),
    });
  },

  /**
   * Delete a specific assignment from a module.
   */
  deleteAssignment: (
    moduleId: string,
    assignmentId: string
  ): Promise<ApiResponse<DeleteAssignmentResponse>> => {
    return apiFetch(`/modules/${moduleId}/assignments/${assignmentId}`, {
      method: "DELETE",
    });
  },

  /**
   * Fetch metadata and file list for a single assignment.
   */
  getAssignmentDetails: (
    moduleId: string,
    assignmentId: string
  ): Promise<ApiResponse<AssignmentDetailsResponse>> => {
    return apiFetch(`/modules/${moduleId}/assignments/${assignmentId}`, {
      method: "GET",
    });
  },

  /**
   * List assignments for a specific module with pagination, filters, and sorting.
   */
  listAssignments: (
    moduleId: string,
    request: ListAssignmentsRequest
  ): Promise<ApiResponse<ListAssignmentsResponse>> => {
    const params = new URLSearchParams();

    params.append("page", request.page.toString());
    params.append("per_page", request.per_page.toString());

    if (Array.isArray(request.sort)) {
      const sortValue = request.sort
        .map(({ field, order }) => (order === "desc" ? `-${field}` : field))
        .join(",");
      if (sortValue) params.append("sort", sortValue);
    }

    if (request.name) params.append("name", request.name);
    if (request.assignment_type) params.append("assignment_type", request.assignment_type);
    if (request.available_before) params.append("available_before", request.available_before);
    if (request.available_after) params.append("available_after", request.available_after);
    if (request.due_before) params.append("due_before", request.due_before);
    if (request.due_after) params.append("due_after", request.due_after);

    return apiFetch(`/modules/${moduleId}/assignments?${params.toString()}`, {
      method: "GET",
    });
  },

  /**
   * Upload one or more files to a specific assignment.
   */
  uploadFiles: (
    moduleId: string,
    assignmentId: string,
    files: File[]
  ): Promise<ApiResponse<UploadAssignmentFilesResponse>> => {
    const form = new FormData();
    for (const file of files) {
      form.append("files[]", file);
    }

    return apiFetch(`/modules/${moduleId}/assignments/${assignmentId}/files`, {
      method: "POST",
      body: form,
    });
  },

  /**
   * List metadata for all files of a specific assignment.
   */
  listFiles: (
    moduleId: string,
    assignmentId: string
  ): Promise<ApiResponse<ListAssignmentFilesResponse>> => {
    return apiFetch(`/modules/${moduleId}/assignments/${assignmentId}/files`, {
      method: "GET",
    });
  },

  /**
   * Delete one or more assignment files by ID.
   */
  deleteFiles: (
    moduleId: string,
    assignmentId: string,
    payload: DeleteAssignmentFilesRequest
  ): Promise<ApiResponse<DeleteAssignmentFilesResponse | PartialDeleteAssignmentFilesResponse>> => {
    return apiFetch(`/modules/${moduleId}/assignments/${assignmentId}/files`, {
      method: "DELETE",
      body: JSON.stringify(payload),
    });
  },

  /**
   * Download a specific file by its ID.
   */
  downloadFile: (
    moduleId: string,
    assignmentId: string,
    fileId: string
  ): Promise<Response> => {
    return fetch(`/api/modules/${moduleId}/assignments/${assignmentId}/files/${fileId}`, {
      method: "GET",
      credentials: "include",
    });
  },
};
