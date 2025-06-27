import { useEffect, useState } from 'react';
import { InputNumber, Select, Typography, Button, Form } from 'antd';
import PageHeader from '@/components/PageHeader';
import SettingsGroup from '@/components/SettingsGroup';
import { useNotifier } from '@/components/Notifier';
import { useModule } from '@/context/ModuleContext';
import { useAssignment } from '@/context/AssignmentContext';
import type { AssignmentConfig } from '@/types/modules/assignments/config';
import { uploadAssignmentFile, downloadAssignmentFile } from '@/services/modules/assignments';

const { Paragraph } = Typography;

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
  const { notifyError, notifySuccess } = useNotifier();
  const [form] = Form.useForm();
  const [loading, setLoading] = useState(true);

  const configFile = assignment.files.find((f) => f.file_type === 'config');
  const configFileId = configFile?.id;

  useEffect(() => {
    form.setFieldsValue(defaultConfig);
    setLoading(false);
  }, []);

  const handleUploadFromDisk = async (e: React.ChangeEvent<HTMLInputElement>) => {
    const file = e.target.files?.[0];
    if (!file) return;

    try {
      const res = await uploadAssignmentFile(module.id, assignment.id, 'config', file);
      if (res.success) {
        notifySuccess('Uploaded', 'Config file uploaded successfully.');
      } else {
        notifyError('Upload failed', res.message);
      }
    } catch (err) {
      console.error(err);
      notifyError('Upload failed', 'Could not upload config file.');
    } finally {
      e.target.value = '';
    }
  };

  const handleDownloadFromServer = async () => {
    if (!configFileId) {
      notifyError('No config file', 'No config file was found on the server.');
      return;
    }

    try {
      await downloadAssignmentFile(module.id, assignment.id, configFileId);
      notifySuccess('Downloaded', 'Config file downloaded from server.');
    } catch (err) {
      console.error(err);
      notifyError('Download failed', 'Could not download config file from server.');
    }
  };

  const handleDownloadLocal = () => {
    const values = form.getFieldsValue() as AssignmentConfig;
    const blob = new Blob([JSON.stringify(values, null, 2)], { type: 'application/json' });
    const url = URL.createObjectURL(blob);
    const link = document.createElement('a');
    link.href = url;
    link.download = `assignment_${assignment.id}_config.json`;
    document.body.appendChild(link);
    link.click();
    document.body.removeChild(link);
  };

  const handleRevert = () => {
    form.setFieldsValue(defaultConfig);
    notifySuccess('Reverted', 'Default configuration restored.');
  };

  const handleSaveAndUpload = async () => {
    const values = form.getFieldsValue() as AssignmentConfig;
    const blob = new Blob([JSON.stringify(values, null, 2)], { type: 'application/json' });
    const file = new File([blob], 'config.json', { type: 'application/json' });

    try {
      const res = await uploadAssignmentFile(module.id, assignment.id, 'config', file);
      if (res.success) {
        notifySuccess('Saved & Uploaded', 'Configuration saved and uploaded.');
      } else {
        notifyError('Upload failed', res.message);
      }
    } catch (err) {
      console.error(err);
      notifyError('Upload failed', 'Could not upload config.');
    }
  };

  return (
    <div className="max-w-4xl">
      <PageHeader
        title="Assignment Configuration"
        description="Manage technical constraints and upload the config JSON to the server."
      />

      {loading ? (
        <Paragraph type="secondary">Loading...</Paragraph>
      ) : (
        <Form layout="vertical" form={form} className="space-y-6">
          <SettingsGroup title="Timeout" description="Max execution time in seconds.">
            <Form.Item name="timeout_secs" noStyle>
              <InputNumber min={1} className="w-40" />
            </Form.Item>
          </SettingsGroup>

          <SettingsGroup title="Max Memory" description='Maximum memory, e.g. "256m".'>
            <Form.Item name="max_memory" noStyle>
              <InputNumber className="w-40" min={1} addonAfter="MB" />
            </Form.Item>
          </SettingsGroup>

          <SettingsGroup title="Max CPUs" description='Maximum CPUs, e.g. "1.5".'>
            <Form.Item name="max_cpus" noStyle>
              <InputNumber min={0.1} step={0.1} className="w-40" />
            </Form.Item>
          </SettingsGroup>

          <SettingsGroup
            title="Max Uncompressed Size"
            description="Limit for extracted archive contents (in bytes)."
          >
            <Form.Item name="max_uncompressed_size" noStyle>
              <InputNumber min={1} className="w-60" addonAfter="bytes" />
            </Form.Item>
          </SettingsGroup>

          <SettingsGroup title="Max Processes" description="Maximum number of processes allowed.">
            <Form.Item name="max_processes" noStyle>
              <InputNumber min={1} className="w-40" />
            </Form.Item>
          </SettingsGroup>

          <SettingsGroup title="Marking Scheme" description="How student output is compared.">
            <Form.Item name="marking_scheme" noStyle>
              <Select
                className="w-60"
                options={[
                  { value: 'exact', label: 'Exact Match' },
                  { value: 'percentage', label: 'Percentage Match' },
                  { value: 'regex', label: 'Regex Match' },
                ]}
              />
            </Form.Item>
          </SettingsGroup>

          <SettingsGroup title="Feedback Scheme" description="How feedback is given.">
            <Form.Item name="feedback_scheme" noStyle>
              <Select
                className="w-60"
                options={[
                  { value: 'auto', label: 'Auto' },
                  { value: 'manual', label: 'Manual' },
                  { value: 'ai', label: 'AI-Assisted' },
                ]}
              />
            </Form.Item>
          </SettingsGroup>

          <div className="pt-4 flex flex-wrap justify-end gap-2">
            <Button onClick={handleRevert}>Revert to Default</Button>
            <Button onClick={handleDownloadLocal}>Download as File</Button>
            <Button onClick={handleDownloadFromServer} disabled={!configFileId}>
              Download from Server
            </Button>
            <Button type="primary" onClick={handleSaveAndUpload}>
              Save & Upload
            </Button>
            <Button onClick={() => document.getElementById('config-upload')?.click()}>
              Upload File
            </Button>
            <input
              type="file"
              id="config-upload"
              hidden
              accept=".json"
              onChange={handleUploadFromDisk}
            />
          </div>
        </Form>
      )}
    </div>
  );
};

export default Config;
