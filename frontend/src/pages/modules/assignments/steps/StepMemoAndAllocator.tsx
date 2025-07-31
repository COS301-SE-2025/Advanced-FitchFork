import { Typography, Button, Steps } from 'antd';
import { useState } from 'react';

import { useModule } from '@/context/ModuleContext';
import { useAssignmentSetup } from '@/context/AssignmentSetupContext';
import { generateMemoOutput } from '@/services/modules/assignments/memo-output';
import { generateMarkAllocator } from '@/services/modules/assignments/mark-allocator';
import { message } from '@/utils/message';

const { Title, Paragraph } = Typography;
const { Step } = Steps;

const StepMemoAndAllocator = () => {
  const module = useModule();
  const { assignmentId, readiness, refreshAssignment, onStepComplete } = useAssignmentSetup();

  const [loading, setLoading] = useState(false);
  const [error, setError] = useState(false);

  // Dynamically compute currentStep from readiness
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
      message.success(resMemo.message);
      await refreshAssignment?.();
    } catch (err: any) {
      console.error(err);
      message.error(err.message || 'Memo output generation failed');
      setLoading(false);
      setError(true);
      return;
    }

    try {
      const resAllocator = await generateMarkAllocator(module.id, assignmentId);
      if (!resAllocator.success) throw new Error(resAllocator.message);
      message.success(resAllocator.message);
      await refreshAssignment?.();
      onStepComplete?.();
    } catch (err: any) {
      console.error(err);
      message.error(err.message || 'Mark allocator generation failed');
      setLoading(false);
      setError(true);
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
