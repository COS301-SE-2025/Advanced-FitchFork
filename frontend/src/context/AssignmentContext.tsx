import type {
  Assignment,
  AssignmentFile,
  AssignmentReadiness,
  AttemptsInfo,
  BestMark,
  AssignmentPolicy,
} from '@/types/modules/assignments';
import type { MarkAllocatorFile } from '@/types/modules/assignments/mark-allocator';
import type { MemoTaskOutput } from '@/types/modules/assignments/memo-output';
import type { AssignmentConfig } from '@/types/modules/assignments/config';
import { createContext, useContext } from 'react';
import type { AssignmentStats } from '@/types/modules/assignments';

export interface AssignmentContextValue {
  assignment: Assignment;
  assignmentFiles: AssignmentFile[];
  memoOutput: MemoTaskOutput[];
  attempts: AttemptsInfo | null;
  bestMark: BestMark | null;
  markAllocator: MarkAllocatorFile | null;
  readiness: AssignmentReadiness | null;

  /** Student-safe subset of config always available in details */
  policy: AssignmentPolicy | null;

  /** Full config (staff/admin only) */
  config: AssignmentConfig | null;

  assignmentStats: AssignmentStats | null;

  incrementAttempts: () => void;

  loading: boolean;
  refreshAssignment: () => Promise<void>;
  refreshAssignmentStats: (days?: number) => Promise<void>;

  // Save partial (provider merges & POSTs full)
  updateConfig: (patch: Partial<AssignmentConfig>) => Promise<void>;

  // NEW: overwrite with system defaults (POST /config/reset)
  resetConfig: () => Promise<void>;
}

const AssignmentContext = createContext<AssignmentContextValue | null>(null);

export const useAssignment = (): AssignmentContextValue => {
  const context = useContext(AssignmentContext);
  if (!context) throw new Error('useAssignment must be used within an AssignmentProvider');
  return context;
};

export const AssignmentProvider = AssignmentContext.Provider;
