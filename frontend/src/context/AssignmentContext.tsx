import type { Assignment, AssignmentFile, AssignmentReadiness } from '@/types/modules/assignments';
import { createContext, useContext } from 'react';

interface AssignmentDetails extends Assignment {
  files: AssignmentFile[];
}

interface AssignmentContextValue {
  assignment: AssignmentDetails;
  readiness: AssignmentReadiness | null;
  refreshReadiness: () => Promise<void>;
}

const AssignmentContext = createContext<AssignmentContextValue | null>(null);

export const useAssignment = (): AssignmentContextValue => {
  const context = useContext(AssignmentContext);
  if (!context) throw new Error('useAssignment must be used within an AssignmentProvider');
  return context;
};

export const AssignmentProvider = AssignmentContext.Provider;
