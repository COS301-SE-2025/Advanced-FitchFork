// src/pages/modules/assignments/steps/AssignmentSteps.tsx

import { Outlet, useLocation } from 'react-router-dom';
import { Steps } from 'antd';
import { StepNavigatorProvider } from '@/context/StepNavigatorContext';
import { useModule } from '@/context/ModuleContext';
import { useAssignment } from '@/context/AssignmentContext';

// Define steps for assignment setup
const ASSIGNMENT_STEPS = [
  'config',
  'main',
  'memo',
  'makefile',
  'tasks',
  'memo-output',
  'mark-allocator',
] as const;

const AssignmentSteps = () => {
  const module = useModule();
  const { assignment } = useAssignment();
  const location = useLocation();

  const basePath = `/modules/${module.id}/assignments/${assignment.id}`;
  const currentIndex = ASSIGNMENT_STEPS.findIndex((step) =>
    location.pathname.startsWith(`${basePath}/${step}`),
  );

  return (
    <StepNavigatorProvider steps={ASSIGNMENT_STEPS as unknown as string[]} basePath={basePath}>
      <div className="w-full overflow-x-auto pb-4">
        <div className="min-w-[700px] px-2">
          <Steps
            current={currentIndex >= 0 ? currentIndex : 0}
            size="small"
            items={ASSIGNMENT_STEPS.map((step) => ({
              title: step.replace(/-/g, ' ').toUpperCase(),
            }))}
          />
        </div>
      </div>
    </StepNavigatorProvider>
  );
};

export default AssignmentSteps;
