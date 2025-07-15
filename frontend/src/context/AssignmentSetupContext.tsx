import { createContext, useContext } from 'react';
import type { Assignment, AssignmentFile, AssignmentReadiness } from '@/types/modules/assignments';

export interface AssignmentDetails extends Assignment {
  files: AssignmentFile[];
}

export interface AssignmentSetupContextValue {
  assignmentId: number | null;
  assignment: AssignmentDetails | null;
  setAssignment: (assignment: AssignmentDetails) => void;
  setStepSaveHandler?: (step: number, handler: () => Promise<boolean>) => void;
  refreshAssignment?: () => Promise<void>;
  readiness?: AssignmentReadiness | null;
}

const AssignmentSetupContext = createContext<AssignmentSetupContextValue | null>(null);

export const useAssignmentSetup = (): AssignmentSetupContextValue => {
  const ctx = useContext(AssignmentSetupContext);
  if (!ctx) throw new Error('useAssignmentSetup must be used within an AssignmentSetupProvider');
  return ctx;
};

export const AssignmentSetupProvider = AssignmentSetupContext.Provider;
