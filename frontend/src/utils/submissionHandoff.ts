// src/utils/submissionHandoff.ts
export type PendingSubmission = {
  moduleId: number;
  assignmentId: number;
  file: File;
  isPractice: boolean;
  attestsOwnership: boolean;
};

let pending: PendingSubmission | null = null;

export function setPendingSubmission(p: PendingSubmission) {
  pending = p;
}

export function takePendingSubmission(): PendingSubmission | null {
  const p = pending;
  pending = null;
  return p;
}
