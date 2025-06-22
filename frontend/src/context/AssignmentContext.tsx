import { createContext, useContext } from 'react';
import type { AssignmentDetailsResponse } from '@/types/assignments';

interface AssignmentContextValue {
  assignment: AssignmentDetailsResponse;
}

const AssignmentContext = createContext<AssignmentContextValue | null>(null);

export const useAssignment = (): AssignmentDetailsResponse => {
  const context = useContext(AssignmentContext);
  if (!context) throw new Error('useAssignment must be used within an AssignmentProvider');
  return context.assignment;
};

export const AssignmentProvider = AssignmentContext.Provider;
