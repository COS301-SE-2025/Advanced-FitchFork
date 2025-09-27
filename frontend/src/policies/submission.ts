// src/policies/submission.ts
import type { SubmissionMode } from '@/types/modules/assignments/config';

/** Whether the Memo Output + Mark Allocator step should be shown at all */
export const showMemoAllocatorForMode = (mode?: SubmissionMode | null): boolean =>
  mode === 'manual';

/** File requirements used by Files & Resources gating */
export const requiresMainForMode = (mode?: SubmissionMode | null): boolean =>
  mode === 'manual';

export const requiresInterpreterForMode = (mode?: SubmissionMode | null): boolean =>
  mode === 'gatlam' || mode === 'rng' || mode === 'codecoverage';
