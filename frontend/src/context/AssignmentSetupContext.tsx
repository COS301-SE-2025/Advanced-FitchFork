import { createContext, useContext } from 'react';
import type { Assignment, AssignmentFile, AssignmentReadiness } from '@/types/modules/assignments';
import type { AssignmentConfig } from '@/types/modules/assignments/config';

export interface AssignmentDetails extends Assignment {
  files: AssignmentFile[];
}

export interface AssignmentSetupContextValue {
  /** Assignment identity */
  assignmentId: number | null;

  /** Local snapshots (wizard-scoped, not the parent page) */
  assignment: AssignmentDetails | null;
  readiness?: AssignmentReadiness | null;
  config: AssignmentConfig | null;

  /** Local setters */
  setAssignment: (assignment: AssignmentDetails) => void;
  setConfig: (config: AssignmentConfig) => void;

  /** Refresh local snapshots (details/readiness/config) */
  refreshAssignment?: () => Promise<void>;

  /** Steps can register a save handler keyed by logical step (1..N) */
  setStepSaveHandler?: (step: number, handler: () => Promise<boolean>) => void;

  /** Navigate the wizard programmatically from any step */
  next: () => Promise<void>; // advances to the next logical step (runs save handler if registered)
  prev: () => void; // goes back one step (never past the first post-welcome step)
}

const AssignmentSetupContext = createContext<AssignmentSetupContextValue | null>(null);

export const useAssignmentSetup = (): AssignmentSetupContextValue => {
  const ctx = useContext(AssignmentSetupContext);
  if (!ctx) throw new Error('useAssignmentSetup must be used within an AssignmentSetupProvider');
  return ctx;
};

export const AssignmentSetupProvider = AssignmentSetupContext.Provider;
