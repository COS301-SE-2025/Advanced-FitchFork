import { api } from "@/utils/api";

export interface SetIgnoredData {
  id: number;
  ignored: boolean;
  updated_at: string;
}

export function setSubmissionIgnored(
  moduleId: number,
  assignmentId: number,
  submissionId: number,
  ignored: boolean
) {
  return api.patch<SetIgnoredData>(
    `/modules/${moduleId}/assignments/${assignmentId}/submissions/${submissionId}/ignore`,
    { ignored }
  );
}