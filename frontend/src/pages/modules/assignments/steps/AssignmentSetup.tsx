// AssignmentSetup.tsx
import { useEffect, useState, useCallback, useMemo } from 'react';
import { Modal, Steps, Button } from 'antd';
import { CheckOutlined, LeftOutlined, RightOutlined } from '@ant-design/icons';

import { AssignmentSetupProvider } from '@/context/AssignmentSetupContext';
import { getAssignmentDetails, getAssignmentReadiness } from '@/services/modules/assignments';
import { getAssignmentConfig } from '@/services/modules/assignments/config';

import StepWelcome from './StepWelcome';
import StepConfig from './StepConfig';
import StepFilesResources from './StepFilesResources';
import StepTasks from './StepTasks';
import StepMemoAndAllocator from './StepMemoAndAllocator';
import StepFinal from './StepFinal';

import type { Module } from '@/types/modules';
import type { AssignmentDetails, AssignmentReadiness } from '@/types/modules/assignments';
import type { AssignmentConfig, SubmissionMode } from '@/types/modules/assignments/config';

const { Step } = Steps;

// Components, not instantiated nodes
const STEPS = [
  { title: 'Welcome', Component: StepWelcome }, // 0
  { title: 'Config', Component: StepConfig }, // 1 (never the default)
  { title: 'Files & Resources', Component: StepFilesResources }, // 2
  { title: 'Tasks', Component: StepTasks }, // 3
  { title: 'Memo & Allocator', Component: StepMemoAndAllocator }, // 4
] as const;

type Props = {
  open: boolean;
  onClose: () => void;
  assignmentId: number;
  module: Module;
  onDone?: () => void;
};

const AssignmentSetup = ({ open, onClose, assignmentId, module, onDone }: Props) => {
  const [current, setCurrent] = useState(0);
  const [assignment, setAssignment] = useState<AssignmentDetails | null>(null);
  const [readiness, setReadiness] = useState<AssignmentReadiness | null>(null);
  const [config, setConfig] = useState<AssignmentConfig | null>(null);

  const [stepSaveHandlers, setStepSaveHandlers] = useState<Record<number, () => Promise<boolean>>>(
    {},
  );

  const setStepSaveHandler = useCallback((step: number, handler: () => Promise<boolean>) => {
    setStepSaveHandlers((prev) => (prev[step] === handler ? prev : { ...prev, [step]: handler }));
  }, []);

  /** unified refresh (kept for child steps calling refresh) */
  const refreshLocal = useCallback(
    async (idOverride?: number) => {
      const idToUse = idOverride ?? assignment?.assignment.id ?? assignmentId;
      if (!idToUse) return;

      const [detailsRes, readinessRes, configRes] = await Promise.all([
        getAssignmentDetails(module.id, idToUse),
        getAssignmentReadiness(module.id, idToUse),
        getAssignmentConfig(module.id, idToUse),
      ]);

      if (detailsRes.success && detailsRes.data) setAssignment(detailsRes.data);
      if (readinessRes.success && readinessRes.data) setReadiness(readinessRes.data);
      if (configRes.success) setConfig(configRes.data);
    },
    [assignment?.assignment.id, assignmentId, module.id],
  );

  /** choose initial step (Welcome by default; never Config) */
  const decideStartStep = (r?: AssignmentReadiness | null): number => {
    if (!r) return 0; // Welcome

    // Final artifacts present → jump to "Memo & Allocator"
    if (r.memo_output_present || r.mark_allocator_present) return 4;

    // Tasks present (but no final artifacts) → "Tasks"
    if (r.tasks_present) return 3;

    // Any files/interpreter present → "Files & Resources"
    if (r.main_present || r.memo_present || r.makefile_present || r.interpreter_present) return 2;

    // Otherwise → Welcome
    return 0;
  };

  /** navigation */
  const next = useCallback(async () => {
    const save = stepSaveHandlers[current];
    if (save) {
      const ok = await save();
      if (!ok) return;
    }
    await refreshLocal(assignment?.assignment.id ?? assignmentId);
    setCurrent((prev) => Math.min(prev + 1, STEPS.length - 1));
  }, [current, stepSaveHandlers, refreshLocal, assignment?.assignment.id, assignmentId]);

  const prev = useCallback(() => setCurrent((p) => Math.max(p - 1, 0)), []);

  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      if (e.key === 'ArrowRight') void next();
      if (e.key === 'ArrowLeft') prev();
    };
    window.addEventListener('keydown', handleKeyDown);
    return () => window.removeEventListener('keydown', handleKeyDown);
  }, [next, prev]);

  // Load + decide start step when opening (don’t rely on async state settling)
  useEffect(() => {
    if (open && assignmentId) {
      (async () => {
        const [detailsRes, readinessRes, configRes] = await Promise.all([
          getAssignmentDetails(module.id, assignmentId),
          getAssignmentReadiness(module.id, assignmentId),
          getAssignmentConfig(module.id, assignmentId),
        ]);

        if (detailsRes.success && detailsRes.data) setAssignment(detailsRes.data);
        const r = readinessRes.success ? readinessRes.data : null;
        if (r) setReadiness(r);
        if (configRes.success) setConfig(configRes.data);

        // Decide the starting step:
        // - 0 Welcome by default
        // - 2 if any files/interpreter
        // - 3 if tasks
        // - 4 if outputs exist
        // - NEVER 1 (Config) by default
        setCurrent(decideStartStep(r));
      })();
    } else if (!open) {
      setCurrent(0);
      setAssignment(null);
      setReadiness(null);
      setConfig(null);
      setStepSaveHandlers({});
    }
  }, [open, assignmentId, module.id]);

  /** completion (controls Final screen) */
  const showFinal = useMemo(() => {
    if (readiness?.is_ready === true) return true;
    if (readiness?.memo_output_present && readiness?.mark_allocator_present) return true;
    return false;
  }, [readiness]);

  /** gating (only when not final) */
  const isCurrentStepComplete = (): boolean => {
    if (showFinal) return true;
    if (current === 0) return true; // Welcome
    if (current === 1) return true; // Config is editable; never blocks

    if (!readiness) return false;

    const r = readiness as AssignmentReadiness & {
      submission_mode?: SubmissionMode;
      interpreter_present?: boolean;
    };
    const mode = r.submission_mode;
    const needsMain = mode === 'manual';
    const needsInterpreter = mode === 'gatlam';

    switch (current) {
      case 2:
        return (
          (needsMain ? r.main_present : needsInterpreter ? r.interpreter_present : true) &&
          r.memo_present &&
          r.makefile_present
        );
      case 3:
        return r.tasks_present;
      case 4:
        return r.memo_output_present && r.mark_allocator_present;
      default:
        return true;
    }
  };

  const fireDoneThenClose = () => {
    try {
      onDone?.();
    } finally {
      onClose();
    }
  };

  const providerValue = useMemo(
    () => ({
      assignmentId: assignment?.assignment.id ?? assignmentId ?? null,
      assignment,
      readiness,
      config,
      setAssignment,
      setConfig,
      setStepSaveHandler,
      refreshAssignment: async () => {
        await refreshLocal(assignment?.assignment.id ?? assignmentId);
      },
      next,
      prev,
    }),
    [
      assignment?.assignment.id,
      assignmentId,
      assignment,
      readiness,
      config,
      setStepSaveHandler,
      refreshLocal,
      next,
      prev,
    ],
  );

  const CurrentComp = STEPS[current].Component;

  return (
    <Modal
      open={open}
      onCancel={fireDoneThenClose}
      footer={null}
      width={1380}
      closable={false}
      maskClosable
      destroyOnHidden={false}
      rootClassName="assignment-setup-modal"
    >
      <AssignmentSetupProvider value={providerValue}>
        <div className="!space-y-10 p-8">
          {/* Hide the stepper on the final screen */}
          {!showFinal && (
            <Steps current={current}>
              {STEPS.map((s) => (
                <Step key={s.title} title={s.title} />
              ))}
            </Steps>
          )}

          <div className="min-h-[250px] bg-transparent border-gray-300 dark:border-gray-700 rounded-lg mb-6">
            {showFinal ? (
              <StepFinal />
            ) : current === 0 ? (
              <StepWelcome onManual={() => setCurrent(1)} />
            ) : (
              <CurrentComp />
            )}
          </div>

          <div className="flex justify-center gap-x-4 pt-4">
            {showFinal ? (
              <Button
                size="large"
                type="primary"
                htmlType="button"
                onClick={fireDoneThenClose}
                icon={<CheckOutlined />}
              >
                Finish & Close
              </Button>
            ) : (
              <>
                {assignment && (
                  <Button size="large" type="default" htmlType="button" onClick={fireDoneThenClose}>
                    Continue Later
                  </Button>
                )}

                <Button
                  size="large"
                  htmlType="button"
                  onClick={prev}
                  disabled={current === 0}
                  icon={<LeftOutlined />}
                >
                  Previous
                </Button>

                <Button
                  size="large"
                  htmlType="button"
                  type={current === STEPS.length - 1 ? 'default' : 'primary'}
                  onClick={current === STEPS.length - 1 ? fireDoneThenClose : next}
                  disabled={current !== STEPS.length - 1 && !isCurrentStepComplete()}
                  icon={current === STEPS.length - 1 ? <CheckOutlined /> : <RightOutlined />}
                >
                  {current === STEPS.length - 1 ? 'Finish & Close' : 'Next'}
                </Button>
              </>
            )}
          </div>
        </div>
      </AssignmentSetupProvider>
    </Modal>
  );
};

export default AssignmentSetup;
