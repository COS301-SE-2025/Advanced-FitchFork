import { useEffect, useState } from 'react';
import { Typography, Form, InputNumber, Select, message } from 'antd';
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
  const { assignmentId, setStepSaveHandler } = useAssignmentSetup();
  const [form] = Form.useForm();
  const [, setLoading] = useState(false);

  useEffect(() => {
    form.setFieldsValue(defaultConfig);

    if (setStepSaveHandler) {
      setStepSaveHandler(2, handleSave); // step index of Config
    }
  }, []);

  const handleSave = async (): Promise<boolean> => {
    if (!assignmentId) {
      message.error('Assignment ID is missing — please create assignment first.');
      return false;
    }

    const values = form.getFieldsValue();
    const blob = new Blob([JSON.stringify(values, null, 2)], { type: 'application/json' });
    const file = new File([blob], 'config.json', { type: 'application/json' });

    setLoading(true);
    try {
      const res = await uploadAssignmentFile(module.id, assignmentId, 'config', file);
      if (res.success) {
        message.success('Configuration uploaded successfully.');
        return true;
      } else {
        message.error(`Upload failed: ${res.message}`);
        return false;
      }
    } catch (err) {
      console.error(err);
      message.error('An unexpected error occurred during upload.');
      return false;
    } finally {
      setLoading(false);
    }
  };

  return (
    <div className="space-y-6">
      <Title level={3}>Assignment Configuration</Title>
      <Paragraph type="secondary">
        Define execution limits and grading behavior for this assignment, then save it to upload the
        configuration to the server.
      </Paragraph>

      <Form form={form} layout="vertical" className="grid grid-cols-1 md:grid-cols-2 gap-6">
        <Form.Item label="Timeout (seconds)" name="timeout_secs">
          <InputNumber min={1} />
        </Form.Item>

        <Form.Item label="Max Memory (e.g. 1024m)" name="max_memory">
          <InputNumber addonAfter="MB" min={1} />
        </Form.Item>

        <Form.Item label="Max CPUs" name="max_cpus">
          <InputNumber min={1} />
        </Form.Item>

        <Form.Item label="Max Uncompressed Size (bytes)" name="max_uncompressed_size">
          <InputNumber min={1} />
        </Form.Item>

        <Form.Item label="Max Processes" name="max_processes">
          <InputNumber min={1} />
        </Form.Item>

        <Form.Item label="Marking Scheme" name="marking_scheme">
          <Select>
            <Select.Option value="exact">Exact Match</Select.Option>
            <Select.Option value="percentage">Percentage Match</Select.Option>
            <Select.Option value="regex">Regex Match</Select.Option>
          </Select>
        </Form.Item>

        <Form.Item label="Feedback Scheme" name="feedback_scheme">
          <Select>
            <Select.Option value="auto">Auto</Select.Option>
            <Select.Option value="manual">Manual</Select.Option>
            <Select.Option value="ai">AI-Assisted</Select.Option>
          </Select>
        </Form.Item>
      </Form>
    </div>
  );
};

export default StepConfig;
