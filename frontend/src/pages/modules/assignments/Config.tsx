import { useEffect, useState } from 'react';
import { InputNumber, Select, Typography, Button, Form } from 'antd';
import PageHeader from '@/components/PageHeader';
import SettingsGroup from '@/components/SettingsGroup';
import { useNotifier } from '@/components/Notifier';
import { getAssignmentConfig } from '@/services/modules/assignments/config';
import { setAssignmentConfig } from '@/services/modules/assignments/config/post';
import { useModule } from '@/context/ModuleContext';
import { useAssignment } from '@/context/AssignmentContext';
import type { AssignmentConfig, Languages } from '@/types/modules/assignments/config';

const { Paragraph } = Typography;

const LANGUAGE_OPTIONS: { value: Languages; label: string }[] = [
  { value: 'python', label: 'Python' },
  { value: 'cpp', label: 'C++' },
  { value: 'java', label: 'Java' },
];

const defaultConfig: AssignmentConfig = {
  timeout_seconds: 10,
  max_memory: 256,
  max_cpus: 1,
  max_uncompressed_size: 1000000,
  max_processors: 1,
  languages: 'python',
};

const Config = () => {
  const module = useModule();
  const assignment = useAssignment();
  const { notifyError, notifySuccess } = useNotifier();
  const [loading, setLoading] = useState(true);
  const [form] = Form.useForm();

  const loadConfig = async () => {
    if (!module.id || !assignment.id) return;
    setLoading(true);

    try {
      const res = await getAssignmentConfig(module.id, assignment.id);
      if (res.success) {
        const isEmpty = !res.data || Object.keys(res.data).length === 0;
        const config = isEmpty ? defaultConfig : res.data;

        form.setFieldsValue(config);

        if (isEmpty) {
          await setAssignmentConfig(module.id, assignment.id, defaultConfig);
          notifySuccess('Config initialized', 'Default configuration created.');
        }
      } else {
        notifyError('Failed to load config', res.message);
      }
    } catch (err) {
      console.error(err);
      notifyError('Failed to load config', 'Unexpected error occurred.');
    } finally {
      setLoading(false);
    }
  };

  useEffect(() => {
    loadConfig();
  }, [module.id, assignment.id]);

  const onSave = async () => {
    const values = form.getFieldsValue() as AssignmentConfig;
    try {
      await setAssignmentConfig(module.id, assignment.id, values);
      notifySuccess('Saved', 'Configuration saved successfully.');
    } catch (err) {
      console.error(err);
      notifyError('Failed to save', 'Could not update configuration.');
    }
  };

  return (
    <div className="max-w-4xl">
      <PageHeader
        title="Assignment Configuration"
        description="Manage technical constraints and settings for the assignment container environment."
      />

      {loading ? (
        <Paragraph type="secondary">Loading...</Paragraph>
      ) : (
        <Form layout="vertical" form={form} className="space-y-6">
          <SettingsGroup title="Timeout" description="Max execution time in seconds.">
            <Form.Item name="timeout_seconds" noStyle>
              <InputNumber min={1} className="w-40" />
            </Form.Item>
          </SettingsGroup>

          <SettingsGroup title="Max Memory" description="Maximum memory in MB.">
            <Form.Item name="max_memory" noStyle>
              <InputNumber min={1} className="w-40" addonAfter="MB" />
            </Form.Item>
          </SettingsGroup>

          <SettingsGroup title="Max CPUs" description="CPU cores available.">
            <Form.Item name="max_cpus" noStyle>
              <InputNumber min={0.1} step={0.1} className="w-40" />
            </Form.Item>
          </SettingsGroup>

          <SettingsGroup
            title="Max Uncompressed Size"
            description="Limit for extracted archive contents (bytes)."
          >
            <Form.Item name="max_uncompressed_size" noStyle>
              <InputNumber min={1} className="w-40" addonAfter="bytes" />
            </Form.Item>
          </SettingsGroup>

          <SettingsGroup title="Max Processes" description="Maximum number of processes allowed.">
            <Form.Item name="max_processors" noStyle>
              <InputNumber min={1} className="w-40" />
            </Form.Item>
          </SettingsGroup>

          <SettingsGroup title="Language" description="Language used for execution.">
            <Form.Item name="languages" noStyle>
              <Select className="w-60" options={LANGUAGE_OPTIONS} />
            </Form.Item>
          </SettingsGroup>

          <div className="pt-4 flex justify-end">
            <Button type="primary" onClick={onSave}>
              Save Configuration
            </Button>
          </div>
        </Form>
      )}
    </div>
  );
};

export default Config;
