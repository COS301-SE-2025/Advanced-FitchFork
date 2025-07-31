import { createContext, useContext } from 'react';

export interface StepNavigatorContextValue {
  goToNextStep: () => void;
  goToStep: (stepRoute: string) => void; // NEW
  currentStep: string;
  steps: string[];
}

export const StepNavigatorContext = createContext<StepNavigatorContextValue | null>(null);

export const useStepNavigator = (): StepNavigatorContextValue => {
  const ctx = useContext(StepNavigatorContext);
  if (!ctx) throw new Error('useStepNavigator must be used inside StepNavigatorContext.Provider');
  return ctx;
};
