// src/context/StepNavigatorContext.tsx

import { createContext, useContext, useMemo } from 'react';
import { useLocation, useNavigate } from 'react-router-dom';

interface StepNavigatorContextProps {
  steps: string[];
  currentStep: string | null;
  goToStep: (step: string) => void;
  goToNextStep: () => void;
  goToPreviousStep: () => void;
}

const StepNavigatorContext = createContext<StepNavigatorContextProps | null>(null);

export const StepNavigatorProvider = ({
  steps,
  basePath,
  children,
}: {
  steps: string[];
  basePath: string;
  children: React.ReactNode;
}) => {
  const location = useLocation();
  const navigate = useNavigate();

  const currentStep = useMemo(() => {
    return steps.find((step) => location.pathname.startsWith(`${basePath}/${step}`)) ?? null;
  }, [location.pathname, steps, basePath]);

  const goToStep = (step: string) => {
    navigate(`${basePath}/${step}`);
  };

  const goToNextStep = () => {
    const index = steps.indexOf(currentStep ?? steps[0]);
    const next = steps[index + 1];
    if (next) goToStep(next);
  };

  const goToPreviousStep = () => {
    const index = steps.indexOf(currentStep ?? steps[0]);
    const prev = steps[index - 1];
    if (prev) goToStep(prev);
  };

  return (
    <StepNavigatorContext.Provider
      value={{ steps, currentStep, goToStep, goToNextStep, goToPreviousStep }}
    >
      {children}
    </StepNavigatorContext.Provider>
  );
};

export const useStepNavigator = () => {
  const ctx = useContext(StepNavigatorContext);
  if (!ctx) throw new Error('useStepNavigator must be used within StepNavigatorProvider');
  return ctx;
};
