import { useEffect, useState } from 'react';
import {
  InputNumber,
  Select,
  Typography,
  Button,
  Form,
  Dropdown,
  Segmented,
  Input,
  Menu,
} from 'antd';
import { DownOutlined } from '@ant-design/icons';

import SettingsGroup from '@/components/SettingsGroup';
import CodeEditor from '@/components/CodeEditor';
import { useModule } from '@/context/ModuleContext';
import { useAssignment } from '@/context/AssignmentContext';
import { getAssignmentConfig, setAssignmentConfig } from '@/services/modules/assignments/config';
import { DEFAULT_ASSIGNMENT_CONFIG } from '@/constants/assignments';
import { message } from '@/utils/message';
import {
  FEEDBACK_SCHEME_OPTIONS,
  MARKING_SCHEME_OPTIONS,
  type AssignmentConfig,
} from '@/types/modules/assignments/config';

const toMB = (bytes: number) => Math.round(bytes / (1024 * 1024));
const toBytes = (mb: number) => Math.round(mb * 1024 * 1024);

const Config = () => {
  const module = useModule();
  const { assignment } = useAssignment();
  const [form] = Form.useForm();

  const [rawView, setRawView] = useState(false);
  const [rawText, setRawText] = useState(JSON.stringify(DEFAULT_ASSIGNMENT_CONFIG, null, 2));
  const [config, setConfig] = useState<AssignmentConfig>(DEFAULT_ASSIGNMENT_CONFIG);
  const [selectedSection, setSelectedSection] = useState('execution');

  useEffect(() => {
    const load = async () => {
      try {
        const res = await getAssignmentConfig(module.id, assignment.id);
        if (res.success && res.data) {
          const cfg = res.data;
          setConfig(cfg);
          form.setFieldsValue(cfg);
        } else {
          throw new Error('No config found');
        }
      } catch {
        message.warning('Could not load config. Using defaults.');
        setConfig(DEFAULT_ASSIGNMENT_CONFIG);
        form.setFieldsValue(DEFAULT_ASSIGNMENT_CONFIG);
      }
    };

    load();
  }, [module.id, assignment.id]);

  const syncFormToRaw = () => {
    const updated = form.getFieldsValue(true) as AssignmentConfig;

    setConfig(updated);
    setRawText(JSON.stringify(updated, null, 2));
  };

  const syncRawToForm = () => {
    try {
      const parsed = JSON.parse(rawText) as AssignmentConfig;
      form.setFieldsValue(parsed);
      setConfig(parsed);
    } catch {
      message.error('Invalid JSON');
    }
  };

  const handleSaveAndUpload = async () => {
    let values: AssignmentConfig;

    if (rawView) {
      try {
        values = JSON.parse(rawText);
        form.setFieldsValue(values);
      } catch {
        message.error('Invalid JSON format');
        return;
      }
    } else {
      values = form.getFieldsValue(true);
    }

    try {
      const res = await setAssignmentConfig(module.id, assignment.id, values);
      if (res.success) message.success('Config saved');
      else message.error(res.message);
    } catch {
      message.error('Failed to save config.');
    }
  };

  const handleRevert = () => {
    form.setFieldsValue(DEFAULT_ASSIGNMENT_CONFIG);
    setRawText(JSON.stringify(DEFAULT_ASSIGNMENT_CONFIG, null, 2));
    setConfig(DEFAULT_ASSIGNMENT_CONFIG);
    message.success('Default config restored.');
  };

  const handleDownloadLocal = () => {
    const blob = new Blob([JSON.stringify(config, null, 2)], { type: 'application/json' });
    const url = URL.createObjectURL(blob);
    const a = document.createElement('a');
    a.href = url;
    a.download = 'config.json';
    a.click();
    URL.revokeObjectURL(url);
  };

  const menuItems = [
    {
      key: 'execution',
      label: (
        <div className="flex justify-between items-center">
          <span>Execution</span>
        </div>
      ),
    },
    {
      key: 'marking',
      label: (
        <div className="flex justify-between items-center">
          <span>Marking & Feedback</span>
        </div>
      ),
    },
  ];

  return (
    <div className="bg-white dark:bg-gray-950 border border-gray-200 dark:border-gray-800 rounded-md flex overflow-hidden">
      {/* Sidebar */}
      {!rawView && (
        <div className="w-[240px] bg-gray-50 dark:bg-gray-900 border-r border-gray-200 dark:border-gray-700 px-2 py-2">
          <Menu
            mode="inline"
            theme="light"
            selectedKeys={[selectedSection]}
            onClick={({ key }) => setSelectedSection(key)}
            items={menuItems}
            className="!bg-transparent !p-0"
            style={{ border: 'none' }}
          />
        </div>
      )}

      {/* Main Content */}
      <div className="flex-1 p-6 space-y-6 max-w-5xl">
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
            minimal={true}
          />
        ) : (
          <Form layout="vertical" form={form} className="space-y-6">
            {selectedSection === 'execution' && (
              <SettingsGroup
                title="Execution Limits"
                description="Control how student code is run and isolated."
              >
                <Form.Item name={['execution', 'timeout_secs']} label="Timeout">
                  <InputNumber min={1} className="w-40" addonAfter="sec" />
                </Form.Item>

                <Form.Item
                  name={['execution', 'max_memory']}
                  label="Max Memory"
                  getValueProps={(value) => ({ value: toMB(value) })}
                  normalize={(value) => toBytes(value)}
                >
                  <InputNumber min={1} className="w-40" addonAfter="MB" />
                </Form.Item>

                <Form.Item name={['execution', 'max_cpus']} label="Max CPUs">
                  <InputNumber min={1} step={1} precision={0} className="w-40" addonAfter="cores" />
                </Form.Item>

                <Form.Item
                  name={['execution', 'max_uncompressed_size']}
                  label="Max Uncompressed Size"
                  getValueProps={(value) => ({ value: toMB(value) })}
                  normalize={(value) => toBytes(value)}
                >
                  <InputNumber min={1} className="w-40" addonAfter="MB" />
                </Form.Item>

                <Form.Item name={['execution', 'max_processes']} label="Max Processes">
                  <InputNumber min={1} className="!w-40" />
                </Form.Item>
              </SettingsGroup>
            )}

            {selectedSection === 'marking' && (
              <SettingsGroup
                title="Marking & Feedback"
                description="Determine how submissions are evaluated and feedback is generated."
              >
                <Form.Item name={['marking', 'marking_scheme']} label="Marking Scheme">
                  <Select className="!w-60" options={MARKING_SCHEME_OPTIONS} />
                </Form.Item>

                <Form.Item name={['marking', 'feedback_scheme']} label="Feedback Scheme">
                  <Select className="!w-60" options={FEEDBACK_SCHEME_OPTIONS} />
                </Form.Item>

                <Form.Item name={['marking', 'deliminator']} label="Delimiter String">
                  <Input className="!w-60" addonBefore="Delimiter" />
                </Form.Item>
              </SettingsGroup>
            )}
          </Form>
        )}

        <div className="flex justify-end gap-2 pt-4">
          <Button onClick={handleRevert}>Revert to Default</Button>
          <Dropdown.Button
            type="primary"
            onClick={handleSaveAndUpload}
            menu={{
              items: [{ key: 'download', label: 'Download as File', onClick: handleDownloadLocal }],
            }}
            icon={<DownOutlined />}
          >
            Save Configuration
          </Dropdown.Button>
        </div>
      </div>
    </div>
  );
};

export default Config;
