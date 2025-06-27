import { useState } from 'react';
import { Typography, Button, Alert } from 'antd';
import { CheckOutlined, WarningOutlined } from '@ant-design/icons';
import { useModule } from '@/context/ModuleContext';
import { useAssignment } from '@/context/AssignmentContext';
import { useStepNavigator } from '@/context/StepNavigatorContext';
import { useNotifier } from '@/components/Notifier';
import { generateMemoOutput } from '@/services/modules/assignments/memo-output';

const { Title, Paragraph } = Typography;

const GenerateMemoOutputStep = () => {
  const module = useModule();
  const { assignment, refreshReadiness, readiness } = useAssignment();
  const { goToStep } = useStepNavigator();
  const { notifyError, notifySuccess } = useNotifier();

  const [loading, setLoading] = useState(false);
  const [done, setDone] = useState(false);

  const handleGenerate = async () => {
    if (!module.id || !assignment.id) return;
    setLoading(true);
    try {
      const res = await generateMemoOutput(module.id, assignment.id);
      if (res.success) {
        setDone(true);
        notifySuccess('Memo output generated', res.message);
        await refreshReadiness?.();
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

  const canGenerate = readiness?.memo_present && readiness?.config_present;

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

      <div className="flex gap-4 justify-end mt-6">
        <Button
          type="default"
          disabled={!canGenerate || done}
          loading={loading}
          onClick={handleGenerate}
        >
          Generate Memo Output
        </Button>
        <Button type="primary" disabled={!done} onClick={() => goToStep('mark-allocator')}>
          Next Step: Mark Allocator
        </Button>
      </div>
    </div>
  );
};

export default GenerateMemoOutputStep;
