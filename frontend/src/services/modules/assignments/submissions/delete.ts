import { api } from '@/utils/api';

export interface BulkDeleteSubmissionsResult {
  deleted: number;
  failed: { id: number; error: string }[];
}

export function deleteSubmission(
  moduleId: number,
  assignmentId: number,
  submissionId: number
) {
  const endpoint = `/modules/${moduleId}/assignments/${assignmentId}/submissions/${submissionId}`;
  return api.delete<null>(endpoint);
}

export function bulkDeleteSubmissions(
  moduleId: number,
  assignmentId: number,
  submissionIds: number[]
) {
  const endpoint = `/modules/${moduleId}/assignments/${assignmentId}/submissions/bulk`;
  return api.delete<BulkDeleteSubmissionsResult>(endpoint, {
    submission_ids: submissionIds,
  });
}
