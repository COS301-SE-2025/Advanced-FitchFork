import { useEffect, useState, useCallback } from 'react';
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

import type { Module } from '@/types/modules';
import type { AssignmentReadiness } from '@/types/modules/assignments';
import type { AssignmentDetails } from '@/context/AssignmentSetupContext';
import type { AssignmentConfig } from '@/types/modules/assignments/config';

const { Step } = Steps;

/** All wizard steps INCLUDING Welcome (index 0) */
const stepsAll = [
  { title: 'Welcome', content: null as React.ReactNode }, // filled in render to pass setCurrent
  { title: 'Config', content: <StepConfig /> },
  { title: 'Files & Resources', content: <StepFilesResources /> },
  { title: 'Tasks', content: <StepTasks /> },
  { title: 'Memo & Allocator', content: <StepMemoAndAllocator /> },
];

type Props = {
  open: boolean;
  onClose: () => void;
  assignmentId: number;
  module: Module;
  /** called whenever the modal closes (X/mask/Continue Later) and on Finish */
  onDone?: () => void;
};

const AssignmentSetup = ({ open, onClose, assignmentId, module, onDone }: Props) => {
  // current: 0 = Welcome, 1..4 map to stepsAll[1..4]
  const [current, setCurrent] = useState(0);
  const [assignment, setAssignment] = useState<AssignmentDetails | null>(null);
  const [readiness, setReadiness] = useState<AssignmentReadiness | null>(null);
  const [config, setConfig] = useState<AssignmentConfig | null>(null);

  const [stepSaveHandlers, setStepSaveHandlers] = useState<Record<number, () => Promise<boolean>>>(
    {},
  );

  const setStepSaveHandler = useCallback((step: number, handler: () => Promise<boolean>) => {
    // step is logical index matching current (0..4)
    setStepSaveHandlers((prev) => (prev[step] === handler ? prev : { ...prev, [step]: handler }));
  }, []);

  /**
   * Determine starting step WITH config step:
   * 0 = Welcome (if literally nothing present)
   * 1 = Config (if config missing)
   * 2 = Files & Resources (if any of main/memo/makefile missing)
   * 3 = Tasks (if tasks missing)
   * 4 = Memo & Allocator (if memo_output or mark_allocator missing)
   * else 4
   */
  const determineStep = (r: AssignmentReadiness): number => {
    const nothing =
      !r.config_present &&
      !r.main_present &&
      !r.memo_present &&
      !r.makefile_present &&
      !r.tasks_present &&
      !r.memo_output_present &&
      !r.mark_allocator_present;

    if (nothing) return 0;
    if (!r.config_present) return 1;
    if (!r.main_present || !r.memo_present || !r.makefile_present) return 2;
    if (!r.tasks_present) return 3;
    if (!r.memo_output_present || !r.mark_allocator_present) return 4;
    return 4;
  };

  /** Local-only refresh for details + readiness + config */
  const refreshLocal = useCallback(
    async (idOverride?: number) => {
      const idToUse = idOverride ?? assignment?.id ?? assignmentId;
      if (!idToUse) return;

      const [detailsRes, readinessRes, configRes] = await Promise.all([
        getAssignmentDetails(module.id, idToUse),
        getAssignmentReadiness(module.id, idToUse),
        getAssignmentConfig(module.id, idToUse),
      ]);

      if (detailsRes.success) {
        setAssignment({ ...detailsRes.data, files: detailsRes.data.files ?? [] });
      }
      if (readinessRes.success) {
        setReadiness(readinessRes.data);
      }
      if (configRes.success) {
        setConfig(configRes.data);
      }
    },
    [assignment?.id, assignmentId, module.id],
  );

  // Exposed to context — now supports advancing from Welcome (0 -> 1) too
  const next = useCallback(async () => {
    const saveHandler = stepSaveHandlers[current];
    if (saveHandler) {
      const ok = await saveHandler();
      if (!ok) return;
    }
    await refreshLocal(assignment?.id ?? assignmentId);
    setCurrent((prev) => Math.min(prev + 1, stepsAll.length - 1)); // cap at 4
  }, [current, stepSaveHandlers, refreshLocal, assignment?.id, assignmentId]);

  // Exposed to context — can go all the way back to Welcome (0)
  const prev = useCallback(() => {
    setCurrent((p) => Math.max(p - 1, 0));
  }, []);

  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      // Allow keyboard navigation from Welcome as well
      if (e.key === 'ArrowRight') void next();
      if (e.key === 'ArrowLeft') prev();
    };
    window.addEventListener('keydown', handleKeyDown);
    return () => window.removeEventListener('keydown', handleKeyDown);
  }, [next, prev]);

  useEffect(() => {
    if (open && assignmentId) {
      (async () => {
        const [detailsRes, readinessRes, configRes] = await Promise.all([
          getAssignmentDetails(module.id, assignmentId),
          getAssignmentReadiness(module.id, assignmentId),
          getAssignmentConfig(module.id, assignmentId),
        ]);

        if (detailsRes.success) {
          setAssignment({ ...detailsRes.data, files: detailsRes.data.files ?? [] });
        }
        if (readinessRes.success) {
          const r = readinessRes.data;
          setReadiness(r);
          setCurrent(determineStep(r)); // 0..4 including Welcome
        } else {
          setCurrent(0);
        }
        if (configRes.success) {
          setConfig(configRes.data);
        } else {
          setConfig(null);
        }
      })();
    } else if (!open) {
      // reset when closed
      setCurrent(0);
      setAssignment(null);
      setReadiness(null);
      setConfig(null);
      setStepSaveHandlers({});
    }
  }, [open, assignmentId, module.id]);

  /** Completion gating per step (with config; indices aligned to current) */
  const isCurrentStepComplete = (): boolean => {
    if (!readiness) return false;
    switch (current) {
      case 0: // Welcome doesn't block; let Next proceed
        return true;
      case 1: // Config
        return readiness.config_present;
      case 2: // Files & Resources
        return readiness.main_present && readiness.memo_present && readiness.makefile_present;
      case 3: // Tasks
        return readiness.tasks_present;
      case 4: // Memo & Allocator
        return readiness.memo_output_present && readiness.mark_allocator_present;
      default:
        return true;
    }
  };

  /** Progress rings for partial steps (Files & Memo/Allocator) — indices unchanged */
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
  const isLast = current === stepsAll.length - 1; // 4

  // call onDone on any close AND on finish
  const fireDoneThenClose = () => {
    try {
      onDone?.();
    } finally {
      onClose();
    }
  };

  const handleFinish = () => fireDoneThenClose();

  return (
    <Modal
      open={open}
      onCancel={fireDoneThenClose}
      footer={null}
      width={1380}
      closable={false}
      maskClosable
      destroyOnHidden={false}
    >
      <AssignmentSetupProvider
        value={{
          assignmentId: assignment?.id ?? assignmentId ?? null,
          assignment,
          readiness,
          config,
          setAssignment,
          setConfig,
          setStepSaveHandler,
          refreshAssignment: async () => {
            await refreshLocal(assignment?.id ?? assignmentId);
          },
          next, // expose to steps
          prev, // expose to steps
        }}
      >
        <div className="!space-y-10 pt-4">
          {current !== 0 && (
            <Steps
              current={current} // Welcome is index 0
              percent={current === 2 || current === 4 ? currentStepPercent : undefined}
            >
              {stepsAll.map((item) => (
                <Step key={item.title} title={item.title} />
              ))}
            </Steps>
          )}

          <div className="min-h-[250px] bg-transparent border-gray-300 dark:border-gray-700 rounded-lg mb-6">
            {current === 0 ? (
              // Pass the manual callback to move to step 1 when chosen
              <StepWelcome onManual={() => setCurrent(1)} />
            ) : (
              stepsAll[current].content
            )}
          </div>

          <div className="flex justify-center gap-x-4 pt-4">
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
              type={isLast ? 'default' : 'primary'}
              onClick={isLast ? handleFinish : next}
              disabled={!isLast && !isCurrentStepComplete()}
              icon={isLast ? <CheckOutlined /> : <RightOutlined />}
            >
              {isLast ? 'Finish & Close' : 'Next'}
            </Button>
          </div>
        </div>
      </AssignmentSetupProvider>
    </Modal>
  );
};

export default AssignmentSetup;
