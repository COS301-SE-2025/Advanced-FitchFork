import { useState } from 'react';
import { Typography, Button, Alert } from 'antd';
import { CheckOutlined, WarningOutlined } from '@ant-design/icons';
import { useModule } from '@/context/ModuleContext';
import { useAssignment } from '@/context/AssignmentContext';
import { useNotifier } from '@/components/Notifier';
import { generateMarkAllocator } from '@/services/modules/assignments/mark-allocator';
import { useNavigate } from 'react-router-dom';

const { Title, Paragraph } = Typography;

const GenerateMarkAllocatorStep = () => {
  const module = useModule();
  const { assignment, refreshReadiness, readiness } = useAssignment();
  const { notifyError, notifySuccess } = useNotifier();
  const navigate = useNavigate();

  const [loading, setLoading] = useState(false);
  const [done, setDone] = useState(false);

  const handleGenerate = async () => {
    if (!module.id || !assignment.id) return;
    setLoading(true);
    try {
      const res = await generateMarkAllocator(module.id, assignment.id);
      if (res.success) {
        notifySuccess('Mark allocator generated', res.message);
        setDone(true);
        await refreshReadiness?.();
      } else {
        notifyError('Failed to generate mark allocator', res.message);
      }
    } catch (err) {
      console.error(err);
      notifyError('Error', 'An unexpected error occurred.');
    } finally {
      setLoading(false);
    }
  };

  const canGenerate = readiness?.memo_output_present;

  return (
    <div className="max-w-3xl space-y-6 px-6 py-8 border border-gray-300 dark:border-gray-700 rounded-lg bg-white dark:bg-black/10">
      <Title level={3}>Generate Mark Allocator</Title>
      <Paragraph type="secondary">
        This step will generate a mark allocation template for each task based on the memo output.
        It is used to assign marks per subsection and task in the final marking process.
      </Paragraph>

      {!canGenerate && (
        <Alert
          type="warning"
          icon={<WarningOutlined />}
          message="Memo output must be generated first."
          showIcon
        />
      )}

      {done && (
        <Alert
          type="success"
          icon={<CheckOutlined />}
          message="Mark allocator successfully generated. You may proceed to the next step."
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
          Generate Mark Allocator
        </Button>
        <Button
          type="primary"
          disabled={!done}
          onClick={() => navigate(`/modules/${module.id}/assignments/${assignment.id}/submissions`)}
        >
          Finish Setup
        </Button>
      </div>
    </div>
  );
};

export default GenerateMarkAllocatorStep;
