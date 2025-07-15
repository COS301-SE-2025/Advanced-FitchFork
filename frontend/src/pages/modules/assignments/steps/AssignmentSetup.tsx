import { useEffect, useState } from 'react';
import { Modal, Steps, Button } from 'antd';
import { CheckOutlined, LeftOutlined, RightOutlined } from '@ant-design/icons';

import { AssignmentSetupProvider } from '@/context/AssignmentSetupContext';
import { useNotifier } from '@/components/Notifier';
import {
  createAssignment,
  getAssignmentDetails,
  getAssignmentReadiness,
} from '@/services/modules/assignments';

import StepWelcome from './StepWelcome';
import StepCreateAssignment from './StepCreateAssignment';
import StepConfig from './StepConfig';
import StepFilesResources from './StepFilesResources';
import StepTasks from './StepTasks';
import StepMemoOutput from './StepMemoOutput';
import StepMarkAllocator from './StepMarkAllocator';

import type { Module } from '@/types/modules';
import type { PostAssignmentRequest, AssignmentReadiness } from '@/types/modules/assignments';
import type { AssignmentDetails } from '@/context/AssignmentSetupContext';

const { Step } = Steps;

const AssignmentSetup = ({
  open,
  onClose,
  assignmentId,
  module,
}: {
  open: boolean;
  onClose: () => void;
  assignmentId?: number;
  module: Module;
}) => {
  const [current, setCurrent] = useState(0);

  const [assignmentDraft, setAssignmentDraft] = useState<PostAssignmentRequest>({
    name: '',
    assignment_type: 'assignment',
    available_from: '',
    due_date: '',
    description: '',
  });

  const [assignment, setAssignment] = useState<AssignmentDetails | null>(null);
  const [readiness, setReadiness] = useState<AssignmentReadiness | null>(null);

  const { notifySuccess, notifyError } = useNotifier();

  const [stepSaveHandlers, setStepSaveHandlers] = useState<Record<number, () => Promise<boolean>>>(
    {},
  );

  const setStepSaveHandler = (step: number, handler: () => Promise<boolean>) => {
    setStepSaveHandlers((prev) => ({ ...prev, [step]: handler }));
  };

  const determineStep = (r: AssignmentReadiness | null) => {
    if (!assignment?.id) return 1; // create step if no assignment yet
    if (!r?.config_present) return 2;
    if (!r?.main_present || !r?.memo_present || !r?.makefile_present) return 3;
    if (!r?.tasks_present) return 4;
    if (!r?.memo_output_present) return 5;
    if (!r?.mark_allocator_present) return 6;
    return 0; // welcome
  };

  const next = async () => {
    const saveHandler = stepSaveHandlers[current];
    if (saveHandler) {
      const ok = await saveHandler();
      if (!ok) return;
    }

    if (current === 1 && !assignment) {
      const res = await createAssignment(module.id, { ...assignmentDraft });
      if (res.success) {
        const newAssignment: AssignmentDetails = { ...res.data, files: [] };
        setAssignment(newAssignment);
        notifySuccess('Assignment created', 'You can now continue setup.');
        await refreshAssignment(newAssignment.id, false);
      } else {
        notifyError('Failed to create assignment', res.message);
        return;
      }
    }

    setCurrent((prev) => Math.min(prev + 1, steps.length - 1));
  };

  const prev = () => setCurrent((prev) => Math.max(prev - 1, 0));

  const refreshAssignment = async (idOverride?: number, updateStep = false) => {
    const idToUse = idOverride ?? assignment?.id;
    if (!idToUse) return;

    const [detailsRes, readinessRes] = await Promise.all([
      getAssignmentDetails(module.id, idToUse),
      getAssignmentReadiness(module.id, idToUse),
    ]);

    if (detailsRes.success) {
      const detailed: AssignmentDetails = {
        ...detailsRes.data,
        files: detailsRes.data.files ?? [],
      };
      setAssignment(detailed);
    }

    if (readinessRes.success) {
      setReadiness(readinessRes.data);
      if (updateStep) {
        const step = determineStep(readinessRes.data);
        setCurrent(step);
      }
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
    if (open) {
      (async () => {
        if (!assignment?.id && assignmentId) {
          await refreshAssignment(assignmentId, true);
        } else if (assignment?.id) {
          await refreshAssignment(assignment.id, true);
        } else {
          setCurrent(1); // default to create if nothing
        }
      })();
    }
  }, [open, assignmentId]);

  const steps = [
    { title: 'Welcome', content: <StepWelcome /> },
    {
      title: 'Create',
      content: <StepCreateAssignment draft={assignmentDraft} setDraft={setAssignmentDraft} />,
      nextLabel: 'Save',
    },
    {
      title: 'Config',
      content: <StepConfig />,
      nextLabel: 'Save',
    },
    { title: 'Files & Resources', content: <StepFilesResources />, nextLabel: 'Next' },
    { title: 'Tasks', content: <StepTasks />, nextLabel: 'Next' },
    { title: 'Memo Output', content: <StepMemoOutput />, nextLabel: 'Next' },
    { title: 'Mark Allocator', content: <StepMarkAllocator />, nextLabel: 'Finish' },
  ];

  const nextButtonLabel =
    current === steps.length - 1 ? 'Finish' : steps[current].nextLabel || 'Next';

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
        }}
      >
        <div className="!space-y-10 pt-4">
          <Steps current={current}>
            {steps.map((item) => (
              <Step key={item.title} title={item.title} />
            ))}
          </Steps>

          <div className="min-h-[250px] bg-transparent border border-dashed border-gray-300 dark:border-gray-700 rounded-lg p-8 mb-6">
            {steps[current].content}
          </div>

          <div className="flex justify-end gap-x-4 pt-4">
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
              onClick={current < steps.length - 1 ? next : onClose}
              icon={current === steps.length - 1 ? <CheckOutlined /> : <RightOutlined />}
            >
              {nextButtonLabel}
            </Button>
          </div>
        </div>
      </AssignmentSetupProvider>
    </Modal>
  );
};

export default AssignmentSetup;
