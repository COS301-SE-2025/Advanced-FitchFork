import type { Assignment, AssignmentFile, AssignmentReadiness } from '@/types/modules/assignments';
import type { MarkAllocatorFile } from '@/types/modules/assignments/mark-allocator';
import type { MemoTaskOutput } from '@/types/modules/assignments/memo-output';
import { createContext, useContext } from 'react';

interface AssignmentDetails extends Assignment {
  files: AssignmentFile[];
}

export interface AssignmentContextValue {
  assignment: AssignmentDetails;
  memoOutput: MemoTaskOutput[];
  markAllocator: MarkAllocatorFile | null;
  readiness: AssignmentReadiness | null;
  loading: boolean;
  refreshAssignment: () => Promise<void>;
}

const AssignmentContext = createContext<AssignmentContextValue | null>(null);

export const useAssignment = (): AssignmentContextValue => {
  const context = useContext(AssignmentContext);
  if (!context) throw new Error('useAssignment must be used within an AssignmentProvider');
  return context;
};

export const AssignmentProvider = AssignmentContext.Provider;
