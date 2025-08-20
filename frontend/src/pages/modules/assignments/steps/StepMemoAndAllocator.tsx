import { useEffect, useState } from 'react';
import { Typography, Button, Steps } from 'antd';

import { useModule } from '@/context/ModuleContext';
import { useAssignmentSetup } from '@/context/AssignmentSetupContext';
import { generateMemoOutput } from '@/services/modules/assignments/memo-output';
import { generateMarkAllocator } from '@/services/modules/assignments/mark-allocator';
import { message } from '@/utils/message';

const { Title, Paragraph } = Typography;
const { Step } = Steps;

const StepMemoAndAllocator = () => {
  const module = useModule();
  const { assignmentId, readiness, refreshAssignment, setStepSaveHandler } = useAssignmentSetup();

  const [loading, setLoading] = useState(false);
  const [error, setError] = useState(false);

  // Register a no-op save handler for step 4 (Memo & Allocator)
  useEffect(() => {
    setStepSaveHandler?.(4, async () => true);
  }, [setStepSaveHandler]);

  // 0 = nothing done, 1 = memo done, 2 = both done
  const currentStep =
    readiness?.memo_output_present && readiness?.mark_allocator_present
      ? 2
      : readiness?.memo_output_present
        ? 1
        : 0;

  const handleGenerate = async () => {
    if (!assignmentId) return;
    setLoading(true);
    setError(false);

    try {
      const resMemo = await generateMemoOutput(module.id, assignmentId);
      if (!resMemo.success) throw new Error(resMemo.message);
      message.success(resMemo.message || 'Memo output generated');
      await refreshAssignment?.(); // reflect memo present
    } catch (err: any) {
      // eslint-disable-next-line no-console
      console.error(err);
      message.error(err?.message || 'Memo output generation failed');
      setError(true);
      setLoading(false);
      return;
    }

    try {
      const resAllocator = await generateMarkAllocator(module.id, assignmentId);
      if (!resAllocator.success) throw new Error(resAllocator.message);
      message.success(resAllocator.message || 'Mark allocator generated');
      await refreshAssignment?.(); // reflect allocator present
    } catch (err: any) {
      // eslint-disable-next-line no-console
      console.error(err);
      message.error(err?.message || 'Mark allocator generation failed');
      setError(true);
      setLoading(false);
      return;
    }

    setLoading(false);
  };

  return (
    <div className="max-w-3xl space-y-6 px-6 py-8 border border-gray-300 dark:border-gray-700 rounded-lg bg-white dark:bg-black/10">
      <Title level={3}>Generate Memo Output & Mark Allocator</Title>
      <Paragraph type="secondary">
        This step runs both stages sequentially: first generating the memo output, then generating
        the mark allocator.
      </Paragraph>

      <Steps direction="vertical" current={currentStep} status={error ? 'error' : undefined}>
        <Step
          title="Memo Output"
          description="Process uploaded memo and config files into reference outputs."
        />
        <Step
          title="Mark Allocator"
          description="Generate mark allocation template based on memo output."
        />
      </Steps>

      <div className="flex justify-end pt-6">
        <Button
          type="primary"
          onClick={handleGenerate}
          loading={loading}
          disabled={currentStep === 2}
          data-cy="generate-memo-mark"
        >
          {currentStep === 2 ? 'Completed' : 'Generate'}
        </Button>
      </div>
    </div>
  );
};

export default StepMemoAndAllocator;
