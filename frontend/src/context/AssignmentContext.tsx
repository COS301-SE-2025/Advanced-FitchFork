import type { AssignmentDetails, AssignmentReadiness } from '@/types/modules/assignments';
import type { MarkAllocatorFile } from '@/types/modules/assignments/mark-allocator';
import type { MemoTaskOutput } from '@/types/modules/assignments/memo-output';
import type { AssignmentConfig } from '@/types/modules/assignments/config';
import { createContext, useContext } from 'react';

export interface AssignmentContextValue {
  assignment: AssignmentDetails;
  memoOutput: MemoTaskOutput[];
  markAllocator: MarkAllocatorFile | null;
  readiness: AssignmentReadiness | null;
  config: AssignmentConfig | null;

  loading: boolean;
  refreshAssignment: () => Promise<void>;

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
