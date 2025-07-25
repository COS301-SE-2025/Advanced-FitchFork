import { useState } from 'react';
import { Typography, Button, Upload, Alert } from 'antd';
import { UploadOutlined } from '@ant-design/icons';
import { uploadAssignmentFile } from '@/services/modules/assignments';
import { useModule } from '@/context/ModuleContext';
import { useAssignmentSetup } from '@/context/AssignmentSetupContext';

const { Title, Paragraph } = Typography;

const defaultConfig = {
  timeout_secs: 10,
  max_memory: '1024m',
  max_cpus: '2',
  max_uncompressed_size: 1000000,
  max_processes: 256,
  marking_scheme: 'exact',
  feedback_scheme: 'auto',
};

const StepConfig = () => {
  const module = useModule();
  const { assignmentId, onStepComplete, refreshAssignment } = useAssignmentSetup();
  const [loading, setLoading] = useState(false);
  const [successMessage, setSuccessMessage] = useState<string | null>(null);

  const uploadDefaultConfig = async () => {
    if (!assignmentId) {
      setSuccessMessage(null);
      return;
    }

    const blob = new Blob([JSON.stringify(defaultConfig, null, 2)], { type: 'application/json' });
    const file = new File([blob], 'config.json', { type: 'application/json' });

    setLoading(true);
    try {
      const res = await uploadAssignmentFile(module.id, assignmentId, 'config', file);
      if (res.success) {
        setSuccessMessage('Default configuration uploaded successfully.');
        await refreshAssignment?.();
        onStepComplete?.();
      } else {
        setSuccessMessage(null);
      }
    } catch {
      setSuccessMessage(null);
    } finally {
      setLoading(false);
    }
  };

  const uploadCustomConfig = async (file: File) => {
    if (!assignmentId) {
      setSuccessMessage(null);
      return false;
    }

    const res = await uploadAssignmentFile(module.id, assignmentId, 'config', file);
    if (res.success) {
      setSuccessMessage('Custom configuration uploaded successfully.');
      await refreshAssignment?.();
      onStepComplete?.();
      return true;
    } else {
      setSuccessMessage(null);
      return false;
    }
  };

  return (
    <div className="space-y-6">
      <Title level={3}>Assignment Configuration</Title>

      <Paragraph type="secondary">
        You can start with the default configuration or upload a custom JSON file. You will still be
        able to edit the configuration later if needed.
      </Paragraph>

      {successMessage && (
        <Alert message={successMessage} type="success" showIcon className="!mb-4" />
      )}

      <div className="flex flex-col md:flex-row gap-4">
        <Button type="primary" onClick={uploadDefaultConfig} loading={loading}>
          Use Default Configuration
        </Button>

        <Upload
          beforeUpload={(file) => {
            uploadCustomConfig(file);
            return false; // prevent default upload
          }}
          accept=".json"
          showUploadList={false}
        >
          <Button icon={<UploadOutlined />}>Upload Custom Configuration</Button>
        </Upload>
      </div>
    </div>
  );
};

export default StepConfig;
