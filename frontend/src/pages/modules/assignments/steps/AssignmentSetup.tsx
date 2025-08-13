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
  // remove Welcome from the stepper flow; it's now standalone
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
  const [current, setCurrent] = useState(0); // 0 = Welcome, 1.. = steps array index + 1
  const [assignment, setAssignment] = useState<AssignmentDetails | null>(null);
  const [readiness, setReadiness] = useState<AssignmentReadiness | null>(null);
  const [stepSaveHandlers, setStepSaveHandlers] = useState<Record<number, () => Promise<boolean>>>(
    {},
  );

  const setStepSaveHandler = (step: number, handler: () => Promise<boolean>) => {
    setStepSaveHandlers((prev) => ({ ...prev, [step]: handler }));
  };

  // map "wizard index" to steps[] index (since current 1 maps to steps[0], etc.)
  const stepIdx = Math.max(0, current - 1);

  const determineStep = (r: AssignmentReadiness): number => {
    // Welcome if absolutely nothing present
    if (
      !r.config_present &&
      !r.main_present &&
      !r.memo_present &&
      !r.makefile_present &&
      !r.tasks_present &&
      !r.memo_output_present &&
      !r.mark_allocator_present
    ) {
      return 0; // Welcome
    }
    // Otherwise start at first unmet step (Config = 1)
    if (!r.config_present) return 1;
    if (!r.main_present || !r.memo_present || !r.makefile_present) return 2;
    if (!r.tasks_present) return 3;
    if (!r.memo_output_present || !r.mark_allocator_present) return 4;
    return 4; // everything done -> show last step (read-only)
  };

  const next = async () => {
    if (current === 0) return; // no next on Welcome
    const saveHandler = stepSaveHandlers[current];
    if (saveHandler) {
      const ok = await saveHandler();
      if (!ok) return;
    }
    if (assignment?.id) {
      await refreshAssignment(assignment.id);
      onStepComplete?.();
    }
    setCurrent((prev) => Math.min(prev + 1, steps.length)); // max = 4 (since we have 4 steps after welcome)
  };

  const prev = () => {
    if (current <= 1) return; // don't go back to Welcome via footer
    setCurrent((prev) => Math.max(prev - 1, 1));
  };

  const refreshAssignment = async (idOverride?: number) => {
    const idToUse = idOverride ?? assignment?.id;
    if (!idToUse) return;
    const [detailsRes, readinessRes] = await Promise.all([
      getAssignmentDetails(module.id, idToUse),
      getAssignmentReadiness(module.id, idToUse),
    ]);
    if (detailsRes.success) {
      setAssignment({ ...detailsRes.data, files: detailsRes.data.files ?? [] });
    }
    if (readinessRes.success) {
      setReadiness(readinessRes.data);
    }
  };

  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      if (current === 0) return; // disable arrows on Welcome
      if (e.key === 'ArrowRight') next();
      if (e.key === 'ArrowLeft') prev();
    };
    window.addEventListener('keydown', handleKeyDown);
    return () => window.removeEventListener('keydown', handleKeyDown);
  }, [current]); // eslint-disable-line

  useEffect(() => {
    if (open && assignmentId) {
      (async () => {
        const [detailsRes, readinessRes] = await Promise.all([
          getAssignmentDetails(module.id, assignmentId),
          getAssignmentReadiness(module.id, assignmentId),
        ]);
        if (detailsRes.success) {
          setAssignment({ ...detailsRes.data, files: detailsRes.data.files ?? [] });
        }
        if (readinessRes.success) {
          setReadiness(readinessRes.data);
          setCurrent(determineStep(readinessRes.data));
        }
      })();
    }
  }, [open, assignmentId]); // eslint-disable-line

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
      let c = 0;
      if (readiness.main_present) c++;
      if (readiness.memo_present) c++;
      if (readiness.makefile_present) c++;
      return Math.round((c / 3) * 100);
    }
    if (current === 4) {
      let c = 0;
      if (readiness.memo_output_present) c++;
      if (readiness.mark_allocator_present) c++;
      return Math.round((c / 2) * 100);
    }
    return 0;
  };

  const currentStepPercent = computeStepPercent();
  const isLast = current === steps.length; // 4 if you have 4 post-welcome steps

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
        {/* WELCOME AS STANDALONE */}
        {current === 0 ? (
          <div className="pt-4">
            <div className="min-h-[250px] bg-transparent border border-dashed border-gray-300 dark:border-gray-700 rounded-lg p-8">
              {/* Pass a handler so user can opt into manual flow */}
              <StepWelcome onManual={() => setCurrent(1)} />
            </div>
          </div>
        ) : (
          // NORMAL WIZARD (no Welcome)
          <div className="!space-y-10 pt-4">
            <Steps
              percent={current === 2 || current === 4 ? currentStepPercent : undefined}
              current={stepIdx + 1}
            >
              {steps.map((item) => (
                <Step key={item.title} title={item.title} />
              ))}
            </Steps>

            <div className="min-h-[250px] bg-transparent border border-dashed border-gray-300 dark:border-gray-700 rounded-lg p-8 mb-6">
              {steps[stepIdx].content}
            </div>

            <div className="flex justify-center gap-x-4 pt-4">
              {assignment && (
                <Button size="large" type="default" onClick={onClose}>
                  Continue Later
                </Button>
              )}

              <Button size="large" onClick={prev} disabled={current <= 1} icon={<LeftOutlined />}>
                Previous
              </Button>

              <Button
                size="large"
                type={isLast ? 'default' : 'primary'}
                onClick={isLast ? onClose : next}
                disabled={!isLast && !isCurrentStepComplete()}
                icon={isLast ? <CheckOutlined /> : <RightOutlined />}
              >
                {isLast ? 'Finish & Close' : 'Next'}
              </Button>
            </div>
          </div>
        )}
      </AssignmentSetupProvider>
    </Modal>
  );
};

export default AssignmentSetup;
