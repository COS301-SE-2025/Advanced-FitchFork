import { useState } from 'react';
import { Typography, Button, Alert } from 'antd';
import { CheckOutlined, WarningOutlined } from '@ant-design/icons';

import { useModule } from '@/context/ModuleContext';
import { useAssignmentSetup } from '@/context/AssignmentSetupContext';
import { useNotifier } from '@/components/common/Notifier';
import { generateMemoOutput } from '@/services/modules/assignments/memo-output';

const { Title, Paragraph } = Typography;

const StepMemoOutput = () => {
  const module = useModule();
  const { assignmentId, refreshAssignment, onStepComplete } = useAssignmentSetup();
  const { notifySuccess, notifyError } = useNotifier();

  const [loading, setLoading] = useState(false);
  const [done, setDone] = useState(false);
  const [canGenerate] = useState(true); // optionally toggle based on readiness

  const handleGenerate = async () => {
    if (!assignmentId) return;

    setLoading(true);
    try {
      const res = await generateMemoOutput(module.id, assignmentId);
      if (res.success) {
        setDone(true);
        notifySuccess('Memo output generated', res.message);
        await refreshAssignment?.();
        onStepComplete?.();
      } else {
        notifyError('Failed to generate memo output', res.message);
      }
    } catch (err) {
      console.error(err);
      notifyError('Error', 'An unexpected error occurred.');
    } finally {
      setLoading(false);
    }
  };

  return (
    <div className="max-w-3xl space-y-6 px-6 py-8 border border-gray-300 dark:border-gray-700 rounded-lg bg-white dark:bg-black/10">
      <Title level={3}>Generate Memo Output</Title>
      <Paragraph type="secondary">
        This step will process the uploaded memo and configuration files to produce a reference
        output for each task. This output is used for grading student submissions.
      </Paragraph>

      {!canGenerate && (
        <Alert
          type="warning"
          icon={<WarningOutlined />}
          message="You must upload both a memo file and a config file before you can generate memo output."
          showIcon
        />
      )}

      {done && (
        <Alert
          type="success"
          icon={<CheckOutlined />}
          message="Memo output successfully generated. You may proceed to the next step."
          showIcon
        />
      )}

      <div className="flex justify-end mt-6">
        <Button
          type="primary"
          onClick={handleGenerate}
          loading={loading}
          disabled={!canGenerate || done}
        >
          Generate Memo Output
        </Button>
      </div>
    </div>
  );
};

export default StepMemoOutput;
