import { useEffect, useState } from 'react';
import { Typography, Steps, Button } from 'antd';
import { useModule } from '@/context/ModuleContext';
import { useAssignmentSetup } from '@/context/AssignmentSetupContext';
import { generateMemoOutput } from '@/services/modules/assignments/memo-output';
import { generateMarkAllocator } from '@/services/modules/assignments/mark-allocator';

const { Title, Paragraph, Text } = Typography;
const { Step } = Steps;

const StepMemoAndAllocator = () => {
  const module = useModule();
  const { assignmentId, readiness, refreshAssignment, setStepSaveHandler } = useAssignmentSetup();

  const [loading, setLoading] = useState(false);
  const [errMemo, setErrMemo] = useState<string | null>(null);
  const [errAlloc, setErrAlloc] = useState<string | null>(null);

  useEffect(() => {
    setStepSaveHandler?.(4, async () => true);
  }, [setStepSaveHandler]);

  const isGatlam = readiness?.submission_mode === 'gatlam';
  const gated = isGatlam && !readiness?.interpreter_present;

  const currentStep =
    readiness?.memo_output_present && readiness?.mark_allocator_present
      ? 2
      : readiness?.memo_output_present
        ? 1
        : 0;

  const handleGenerate = async () => {
    if (!assignmentId) return;
    setLoading(true);
    setErrMemo(null);
    setErrAlloc(null);
    try {
      const m = await generateMemoOutput(module.id, assignmentId);
      if (!m.success) setErrMemo(m.message || 'Memo generation failed');
      await refreshAssignment?.();
    } catch (e: any) {
      setErrMemo(e?.message || 'Memo generation failed');
      setLoading(false);
      return;
    }

    try {
      const a = await generateMarkAllocator(module.id, assignmentId);
      if (!a.success) setErrAlloc(a.message || 'Mark allocator failed');
      await refreshAssignment?.();
    } catch (e: any) {
      setErrAlloc(e?.message || 'Mark allocator failed');
      setLoading(false);
      return;
    }
    setLoading(false);
  };

  return (
    <div className="max-w-3xl space-y-6 px-6 py-8 border border-gray-300 dark:border-gray-700 rounded-lg bg-white dark:bg-black/10">
      <Title level={3}>Memo Output & Mark Allocator</Title>
      <Paragraph type="secondary">Run both stages sequentially. Status is shown below.</Paragraph>

      <Steps
        direction="vertical"
        current={currentStep}
        status={errMemo || errAlloc ? 'error' : undefined}
      >
        <Step
          title="Memo Output"
          description={
            <Text type={errMemo ? 'danger' : 'secondary'}>
              {errMemo ? errMemo : readiness?.memo_output_present ? 'Generated' : 'Pending'}
            </Text>
          }
        />
        <Step
          title="Mark Allocator"
          description={
            <Text type={errAlloc ? 'danger' : 'secondary'}>
              {errAlloc ? errAlloc : readiness?.mark_allocator_present ? 'Generated' : 'Pending'}
            </Text>
          }
        />
      </Steps>

      <div className="flex justify-end pt-6">
        <Button
          type="primary"
          onClick={handleGenerate}
          loading={loading}
          disabled={currentStep === 2 || gated}
        >
          {currentStep === 2 ? 'Completed' : 'Run'}
        </Button>
      </div>
    </div>
  );
};

export default StepMemoAndAllocator;
