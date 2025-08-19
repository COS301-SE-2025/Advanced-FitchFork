import { useState } from 'react';
import { Typography, Button, Upload } from 'antd';
import { UploadOutlined, ArrowRightOutlined } from '@ant-design/icons';
import { uploadAssignmentFile } from '@/services/modules/assignments';
import { useModule } from '@/context/ModuleContext';
import { useAssignmentSetup } from '@/context/AssignmentSetupContext';
import { message } from '@/utils/message';

const { Title, Paragraph } = Typography;

const StepConfig = () => {
  const module = useModule();
  const { assignmentId, refreshAssignment, next } = useAssignmentSetup();
  const [loading, setLoading] = useState(false);

  const continueWithCurrentConfig = () => {
    // Welcome already applied defaults + selections
    next?.();
  };

  const uploadCustomConfig = async (file: File) => {
    if (!assignmentId) return false;

    setLoading(true);
    try {
      const res = await uploadAssignmentFile(module.id, assignmentId, 'config', file);
      if (res.success) {
        message.success('Custom configuration uploaded.');
        await refreshAssignment?.();
        next?.(); // advance after successful upload
        return true;
      } else {
        message.error(res.message || 'Failed to upload configuration');
        return false;
      }
    } catch (e: any) {
      message.error(e?.message || 'Failed to upload configuration');
      return false;
    } finally {
      setLoading(false);
    }
  };

  return (
    <div className="space-y-6">
      <Title level={3}>Assignment Configuration</Title>

      <Paragraph type="secondary">
        The basic configuration was already applied on the Welcome step. Continue as-is or upload a
        custom <code>.json</code> config.
      </Paragraph>

      <div className="flex flex-col md:flex-row gap-4">
        <Button
          type="primary"
          onClick={continueWithCurrentConfig}
          loading={loading}
          disabled={!assignmentId || loading}
          icon={<ArrowRightOutlined />}
          data-cy="step-config-continue"
        >
          Use Default Config & Continue
        </Button>

        <Upload
          beforeUpload={(file) => {
            void uploadCustomConfig(file);
            return false; // prevent default upload
          }}
          accept=".json,application/json"
          showUploadList={false}
          disabled={!assignmentId || loading}
        >
          <Button icon={<UploadOutlined />} disabled={!assignmentId || loading}>
            Upload Custom Configuration
          </Button>
        </Upload>
      </div>
    </div>
  );
};

export default StepConfig;
