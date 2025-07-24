import { useEffect, useState } from 'react';
import { Modal, Steps, Button } from 'antd';
import { CheckOutlined, LeftOutlined, RightOutlined } from '@ant-design/icons';

import { AssignmentSetupProvider } from '@/context/AssignmentSetupContext';
import { getAssignmentDetails, getAssignmentReadiness } from '@/services/modules/assignments';

import StepWelcome from './StepWelcome';
import StepConfig from './StepConfig';
import StepFilesResources from './StepFilesResources';
import StepTasks from './StepTasks';
import StepMemoAndAllocator from './StepMemoAndAllocator';

import type { Module } from '@/types/modules';
import type { AssignmentReadiness } from '@/types/modules/assignments';
import type { AssignmentDetails } from '@/context/AssignmentSetupContext';

const { Step } = Steps;

const steps = [
  { title: 'Welcome', content: <StepWelcome /> },
  { title: 'Config', content: <StepConfig /> },
  { title: 'Files & Resources', content: <StepFilesResources /> },
  { title: 'Tasks', content: <StepTasks /> },
  { title: 'Memo & Allocator', content: <StepMemoAndAllocator /> },
];

const AssignmentSetup = ({
  open,
  onClose,
  assignmentId,
  module,
  onStepComplete,
}: {
  open: boolean;
  onClose: () => void;
  assignmentId?: number;
  module: Module;
  onStepComplete?: () => void;
}) => {
  const [current, setCurrent] = useState(0);
  const [assignment, setAssignment] = useState<AssignmentDetails | null>(null);
  const [readiness, setReadiness] = useState<AssignmentReadiness | null>(null);
  const [stepSaveHandlers, setStepSaveHandlers] = useState<Record<number, () => Promise<boolean>>>(
    {},
  );

  const setStepSaveHandler = (step: number, handler: () => Promise<boolean>) => {
    setStepSaveHandlers((prev) => ({ ...prev, [step]: handler }));
  };

  const determineStep = (r: AssignmentReadiness): number => {
    if (
      !r.config_present &&
      !r.main_present &&
      !r.memo_present &&
      !r.makefile_present &&
      !r.tasks_present &&
      !r.memo_output_present &&
      !r.mark_allocator_present
    ) {
      return 0; // Welcome if nothing is present
    }

    if (!r.config_present) return 1;
    if (!r.main_present || !r.memo_present || !r.makefile_present) return 2;
    if (!r.tasks_present) return 3;
    if (!r.memo_output_present || !r.mark_allocator_present) return 4;

    return 0; // fallback
  };

  const next = async () => {
    if (current === 0) {
      setCurrent(1);
      return;
    }

    const saveHandler = stepSaveHandlers[current];
    if (saveHandler) {
      const ok = await saveHandler();
      if (!ok) return;
    }

    if (assignment?.id) {
      await refreshAssignment(assignment.id);
      if (onStepComplete) onStepComplete();
    }

    if (current === steps.length - 1) {
      onClose();
    } else {
      setCurrent((prev) => Math.min(prev + 1, steps.length - 1));
    }
  };

  const prev = () => setCurrent((prev) => Math.max(prev - 1, 0));

  const refreshAssignment = async (idOverride?: number) => {
    const idToUse = idOverride ?? assignment?.id;
    if (!idToUse) return;

    const [detailsRes, readinessRes] = await Promise.all([
      getAssignmentDetails(module.id, idToUse),
      getAssignmentReadiness(module.id, idToUse),
    ]);

    if (detailsRes.success) {
      setAssignment({
        ...detailsRes.data,
        files: detailsRes.data.files ?? [],
      });
    }

    if (readinessRes.success) {
      setReadiness(readinessRes.data);
    }
  };

  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      if (e.key === 'ArrowRight') next();
      if (e.key === 'ArrowLeft') prev();
    };

    window.addEventListener('keydown', handleKeyDown);
    return () => window.removeEventListener('keydown', handleKeyDown);
  }, [current]);

  useEffect(() => {
    if (open && assignmentId) {
      (async () => {
        const [detailsRes, readinessRes] = await Promise.all([
          getAssignmentDetails(module.id, assignmentId),
          getAssignmentReadiness(module.id, assignmentId),
        ]);

        if (detailsRes.success) {
          setAssignment({
            ...detailsRes.data,
            files: detailsRes.data.files ?? [],
          });
        }

        if (readinessRes.success) {
          setReadiness(readinessRes.data);
          setCurrent(determineStep(readinessRes.data)); // only here
        }
      })();
    }
  }, [open, assignmentId]);

  const isCurrentStepComplete = (): boolean => {
    if (!readiness) return false;

    switch (current) {
      case 1:
        return readiness.config_present;
      case 2:
        return readiness.main_present && readiness.memo_present && readiness.makefile_present;
      case 3:
        return readiness.tasks_present;
      case 4:
        return readiness.memo_output_present && readiness.mark_allocator_present;
      default:
        return true;
    }
  };

  const computeStepPercent = (): number => {
    if (!readiness) return 0;

    if (current === 2) {
      let count = 0;
      if (readiness.main_present) count++;
      if (readiness.memo_present) count++;
      if (readiness.makefile_present) count++;
      return Math.round((count / 3) * 100);
    }

    if (current === 4) {
      let count = 0;
      if (readiness.memo_output_present) count++;
      if (readiness.mark_allocator_present) count++;
      return Math.round((count / 2) * 100);
    }

    return 0;
  };

  const currentStepPercent = computeStepPercent();

  return (
    <Modal open={open} onCancel={onClose} footer={null} width={1380} closable={false}>
      <AssignmentSetupProvider
        value={{
          assignmentId: assignment?.id ?? null,
          assignment,
          setAssignment,
          setStepSaveHandler,
          refreshAssignment,
          readiness,
          onStepComplete,
        }}
      >
        <div className="!space-y-10 pt-4">
          <Steps
            percent={current === 2 || current === 4 ? currentStepPercent : undefined}
            current={current}
          >
            {steps.map((item) => (
              <Step key={item.title} title={item.title} />
            ))}
          </Steps>

          <div className="min-h-[250px] bg-transparent border border-dashed border-gray-300 dark:border-gray-700 rounded-lg p-8 mb-6">
            {steps[current].content}
          </div>

          <div className="flex justify-center gap-x-4 pt-4">
            {assignment && (
              <Button size="large" type="default" onClick={onClose}>
                Continue Later
              </Button>
            )}

            <Button size="large" onClick={prev} disabled={current === 0} icon={<LeftOutlined />}>
              Previous
            </Button>

            <Button
              size="large"
              type="primary"
              onClick={next}
              disabled={!isCurrentStepComplete()}
              icon={current === steps.length - 1 ? <CheckOutlined /> : <RightOutlined />}
            >
              {current === steps.length - 1 ? 'Finish' : 'Next'}
            </Button>
          </div>
        </div>
      </AssignmentSetupProvider>
    </Modal>
  );
};

export default AssignmentSetup;
