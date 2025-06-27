import { useNavigate, useLocation, Outlet } from 'react-router-dom';
import { Steps, Typography } from 'antd';
import { useAssignment } from '@/context/AssignmentContext';
import { useModule } from '@/context/ModuleContext';
import { StepNavigatorContext } from '@/context/StepNavigatorContext';

const { Title, Paragraph } = Typography;

const AssignmentSteps = () => {
  const module = useModule();
  const { readiness, assignment } = useAssignment();
  const location = useLocation();
  const navigate = useNavigate();

  const steps = [
    { title: 'Config', route: 'config', done: readiness?.config_present },
    { title: 'Main Files', route: 'main', done: readiness?.main_present },
    { title: 'Memo File', route: 'memo', done: readiness?.memo_present },
    { title: 'Makefile', route: 'makefile', done: readiness?.makefile_present },
    { title: 'Tasks', route: 'tasks', done: readiness?.tasks_present },
    { title: 'Memo Output', route: 'memo-output', done: readiness?.memo_output_present },
    { title: 'Mark Allocator', route: 'mark-allocator', done: readiness?.mark_allocator_present },
  ];

  const stepRoutes = steps.map((s) => s.route);
  const currentStep =
    stepRoutes.find((r) => location.pathname.includes(`/steps/${r}`)) ?? stepRoutes[0];
  const currentIndex = stepRoutes.indexOf(currentStep);

  const goToNextStep = () => {
    const next = stepRoutes[currentIndex + 1];
    if (next) {
      navigate(`/modules/${module.id}/assignments/${assignment.id}/steps/${next}`);
    }
  };

  const goToStep = (route: string) => {
    navigate(`/modules/${module.id}/assignments/${assignment.id}/steps/${route}`);
  };

  return (
    <div className="p-6 space-y-8 max-w-6xl">
      <div className="space-y-2">
        <Title level={3} className="!mb-0 !text-gray-900 dark:!text-gray-100">
          Assignment Setup Progress
        </Title>
        <Paragraph type="secondary" className="!text-gray-700 dark:!text-gray-300">
          Follow the steps below to prepare your assignment. Each step must be completed before the
          assignment can be marked as ready. You can navigate directly to any step to edit or review
          its contents.
        </Paragraph>
      </div>

      <Steps
        current={currentIndex}
        onChange={(i) =>
          navigate(`/modules/${module.id}/assignments/${assignment.id}/steps/${stepRoutes[i]}`)
        }
        items={steps.map((step) => ({
          title: step.title,
          status: step.done ? 'finish' : undefined,
        }))}
        responsive
        className="!mb-6"
      />

      <StepNavigatorContext.Provider
        value={{
          goToNextStep,
          goToStep,
          currentStep,
          steps: stepRoutes,
        }}
      >
        <Outlet />
      </StepNavigatorContext.Provider>
    </div>
  );
};

export default AssignmentSteps;
