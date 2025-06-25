import type { Assignment, AssignmentFile } from '@/types/modules/assignments';
import { createContext, useContext } from 'react';

interface AssignmentDetails extends Assignment {
  files: AssignmentFile[];
}

interface AssignmentContextValue {
  assignment: AssignmentDetails;
}

const AssignmentContext = createContext<AssignmentContextValue | null>(null);

export const useAssignment = (): AssignmentDetails => {
  const context = useContext(AssignmentContext);
  if (!context) throw new Error('useAssignment must be used within an AssignmentProvider');
  return context.assignment;
};

export const AssignmentProvider = AssignmentContext.Provider;
