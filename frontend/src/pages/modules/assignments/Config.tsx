import { useEffect, useState } from 'react';
import { InputNumber, Select, Typography, Button, Form, Dropdown, Segmented, Divider } from 'antd';
import { DownOutlined } from '@ant-design/icons';
import SettingsGroup from '@/components/SettingsGroup';
import { useModule } from '@/context/ModuleContext';
import { useAssignment } from '@/context/AssignmentContext';
import type { AssignmentConfig } from '@/types/modules/assignments/config';
import { uploadAssignmentFile, downloadAssignmentFile } from '@/services/modules/assignments';
import CodeEditor from '@/components/CodeEditor';
import { message } from '@/utils/message';

const defaultConfig: AssignmentConfig = {
  timeout_secs: 10,
  max_memory: '1024m',
  max_cpus: '2',
  max_uncompressed_size: 1000000,
  max_processes: 256,
  marking_scheme: 'exact',
  feedback_scheme: 'auto',
};

const Config = () => {
  const module = useModule();
  const { assignment } = useAssignment();
  const [form] = Form.useForm();

  const [rawView, setRawView] = useState(false);
  const [configFileId, setConfigFileId] = useState<number | null>(null);
  const [rawText, setRawText] = useState<string>(JSON.stringify(defaultConfig, null, 2));

  useEffect(() => {
    const file = assignment.files.find((f) => f.file_type === 'config');
    setConfigFileId(file?.id ?? null);
    form.setFieldsValue(defaultConfig);
    setRawText(JSON.stringify(defaultConfig, null, 2));
  }, [assignment.files]);

  const syncFormToRaw = () => {
    const values = form.getFieldsValue() as AssignmentConfig;
    setRawText(JSON.stringify(values, null, 2));
  };

  const syncRawToForm = () => {
    try {
      const parsed = JSON.parse(rawText);
      form.setFieldsValue(parsed);
    } catch {
      message.error('Invalid JSON. Fix JSON syntax before switching back.');
    }
  };

  const handleSaveAndUpload = async () => {
    let values: AssignmentConfig;

    if (rawView) {
      try {
        values = JSON.parse(rawText);
        form.setFieldsValue(values);
      } catch {
        message.error('Invalid JSON. Please fix the JSON format before saving.');
        return;
      }
    } else {
      values = form.getFieldsValue() as AssignmentConfig;
      setRawText(JSON.stringify(values, null, 2));
    }

    const blob = new Blob([JSON.stringify(values, null, 2)], { type: 'application/json' });
    const file = new File([blob], 'config.json', { type: 'application/json' });
    try {
      const res = await uploadAssignmentFile(module.id, assignment.id, 'config', file);
      if (res.success) message.success('Configuration saved and uploaded.');
      else message.error(`Upload failed: ${res.message}`);
    } catch {
      message.error('Upload failed. Could not upload config.');
    }
  };

  const handleRevert = () => {
    form.setFieldsValue(defaultConfig);
    setRawText(JSON.stringify(defaultConfig, null, 2));
    message.success('Default configuration restored.');
  };

  const handleDownloadFromServer = async () => {
    if (!configFileId) {
      message.error('No config file found on the server.');
      return;
    }
    try {
      await downloadAssignmentFile(module.id, assignment.id, configFileId);
      message.success('Config file downloaded from server.');
    } catch {
      message.error('Could not download config file.');
    }
  };

  const handleDownloadLocal = () => {
    const blob = new Blob([rawText], { type: 'application/json' });
    const url = URL.createObjectURL(blob);
    const link = document.createElement('a');
    link.href = url;
    link.download = `config.json`;
    document.body.appendChild(link);
    link.click();
    document.body.removeChild(link);
  };

  const menuItems = [
    { key: 'downloadLocal', label: 'Download as File', onClick: handleDownloadLocal },
    { key: 'downloadServer', label: 'Download from Server', onClick: handleDownloadFromServer },
  ];

  return (
    <div className="bg-white dark:bg-gray-950 border border-gray-200 dark:border-gray-800 rounded-md p-6 space-y-6">
      <div className="flex justify-between items-center flex-wrap gap-2">
        <div className="flex items-center gap-4">
          <Typography.Title level={4} className="!mb-0">
            Assignment Configuration
          </Typography.Title>
          <Segmented
            options={[
              { label: 'Editor', value: 'form' },
              { label: 'JSON', value: 'raw' },
            ]}
            value={rawView ? 'raw' : 'form'}
            onChange={(val) => {
              if (val === 'raw') {
                syncFormToRaw();
                setRawView(true);
              } else {
                syncRawToForm();
                setRawView(false);
              }
            }}
            size="small"
          />
        </div>
      </div>

      <Typography.Paragraph type="secondary" className="!mt-0">
        Configure how this assignment will be evaluated, including resource limits and marking
        rules. You can use the interactive editor or edit the raw JSON directly.
      </Typography.Paragraph>

      {rawView ? (
        <CodeEditor
          title="Config"
          value={rawText}
          onChange={(val) => setRawText(val ?? '')}
          language="json"
        />
      ) : (
        <Form layout="vertical" form={form} className="space-y-6">
          <SettingsGroup
            title="Execution Limits"
            description="Define resource and execution constraints for submitted assignments. These settings control how much time, memory, CPU, and disk usage are allowed during automated evaluation, and help prevent runaway or abusive processes."
          >
            <Form.Item name="timeout_secs" label="Timeout (seconds)">
              <InputNumber min={1} className="w-40" />
            </Form.Item>
            <Form.Item name="max_memory" label="Max Memory (MB)">
              <InputNumber min={1} className="w-40" />
            </Form.Item>
            <Form.Item name="max_cpus" label="Max CPUs">
              <InputNumber min={0.1} step={0.1} className="w-40" />
            </Form.Item>
            <Form.Item name="max_uncompressed_size" label="Max Uncompressed Size (bytes)">
              <InputNumber min={1} className="w-60" />
            </Form.Item>
            <Form.Item name="max_processes" label="Max Processes">
              <InputNumber min={1} className="w-40" />
            </Form.Item>
          </SettingsGroup>
          <Divider className="!mb-8" />
          <SettingsGroup
            title="Marking & Feedback"
            description="Choose how submitted assignments are scored and how feedback is generated. You can select the marking strategy (exact, percentage, or regex matching) and determine whether feedback is generated automatically, manually, or with AI assistance."
          >
            <Form.Item name="marking_scheme" label="Marking Scheme">
              <Select
                className="!w-60"
                options={[
                  { value: 'exact', label: 'Exact Match' },
                  { value: 'percentage', label: 'Percentage Match' },
                  { value: 'regex', label: 'Regex Match' },
                ]}
              />
            </Form.Item>
            <Form.Item name="feedback_scheme" label="Feedback Scheme">
              <Select
                className="!w-60"
                options={[
                  { value: 'auto', label: 'Auto' },
                  { value: 'manual', label: 'Manual' },
                  { value: 'ai', label: 'AI-Assisted' },
                ]}
              />
            </Form.Item>
          </SettingsGroup>
        </Form>
      )}

      <div className="flex justify-end gap-2 pt-4">
        <Button onClick={handleRevert}>Revert to Default</Button>
        <Dropdown.Button
          type="primary"
          onClick={handleSaveAndUpload}
          menu={{ items: menuItems }}
          icon={<DownOutlined />}
        >
          Save & Upload
        </Dropdown.Button>
      </div>
    </div>
  );
};

export default Config;
