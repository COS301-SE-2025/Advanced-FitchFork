import { useEffect, useState } from 'react';
import { Typography, Form, Segmented, Menu } from 'antd';
import { Link, Outlet, useLocation } from 'react-router-dom';

import CodeEditor from '@/components/common/CodeEditor';

import { useModule } from '@/context/ModuleContext';
import { useAssignment } from '@/context/AssignmentContext';
import { getAssignmentConfig, setAssignmentConfig } from '@/services/modules/assignments/config';
import { DEFAULT_ASSIGNMENT_CONFIG } from '@/constants/assignments';
import type { AssignmentConfig } from '@/types/modules/assignments/config';
import { message } from '@/utils/message';
import type { AssignmentConfigCtx } from '@/context/AssignmentConfigContext';

const ConfigLayout = () => {
  const module = useModule();
  const { assignment } = useAssignment();
  const [form] = Form.useForm<AssignmentConfig>();

  const [rawView, setRawView] = useState(false);
  const [rawText, setRawText] = useState(JSON.stringify(DEFAULT_ASSIGNMENT_CONFIG, null, 2));
  const [config, setConfig] = useState<AssignmentConfig>(DEFAULT_ASSIGNMENT_CONFIG);
  const [loading, setLoading] = useState(false);

  const location = useLocation();
  const selectedKey: 'execution' | 'marking' = location.pathname.endsWith('/marking')
    ? 'marking'
    : 'execution';

  // Load config
  useEffect(() => {
    (async () => {
      setLoading(true);
      try {
        const res = await getAssignmentConfig(module.id, assignment.id);
        if (res.success && res.data) {
          setConfig(res.data);
          form.setFieldsValue(res.data);
          setRawText(JSON.stringify(res.data, null, 2));
        } else {
          throw new Error('No config found');
        }
      } catch {
        message.warning('Could not load config. Using defaults.');
        setConfig(DEFAULT_ASSIGNMENT_CONFIG);
        form.setFieldsValue(DEFAULT_ASSIGNMENT_CONFIG);
        setRawText(JSON.stringify(DEFAULT_ASSIGNMENT_CONFIG, null, 2));
      } finally {
        setLoading(false);
      }
    })();
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

  const save = async () => {
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
      if (res.success) {
        message.success('Config saved');
        setConfig(values);
        setRawText(JSON.stringify(values, null, 2));
      } else {
        message.error(res.message);
      }
    } catch {
      message.error('Failed to save config.');
    }
  };

  const revert = () => {
    form.setFieldsValue(DEFAULT_ASSIGNMENT_CONFIG);
    setRawText(JSON.stringify(DEFAULT_ASSIGNMENT_CONFIG, null, 2));
    setConfig(DEFAULT_ASSIGNMENT_CONFIG);
    message.success('Default config restored.');
  };

  const download = () => {
    const blob = new Blob([JSON.stringify(config, null, 2)], { type: 'application/json' });
    const url = URL.createObjectURL(blob);
    const a = document.createElement('a');
    a.href = url;
    a.download = 'config.json';
    a.click();
    URL.revokeObjectURL(url);
  };

  const outletCtx: AssignmentConfigCtx = {
    form,
    rawView,
    setRawView,
    rawText,
    setRawText,
    loading,
    save,
    revert,
    download,
    syncFormToRaw,
    syncRawToForm,
  };

  const menuItems = [
    { key: 'execution', label: <Link to="execution">Execution</Link> },
    { key: 'marking', label: <Link to="marking">Marking & Feedback</Link> },
  ];

  return (
    <div>
      {/* Desktop / tablet */}
      <div className="hidden sm:flex bg-white dark:bg-gray-900 border rounded-md border-gray-200 dark:border-gray-800 overflow-hidden">
        {/* Sidebar (hidden in JSON mode) */}
        {!rawView && (
          <div className="w-[240px] bg-gray-50 dark:bg-gray-950 border-r border-gray-200 dark:border-gray-800 px-2 py-2">
            <Menu
              mode="inline"
              selectedKeys={[selectedKey]}
              items={menuItems}
              className="!bg-transparent !p-0"
              style={{ border: 'none' }}
            />
          </div>
        )}

        {/* Main */}
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
            Configure execution limits and marking rules. Edit via the sidebar editor or raw JSON.
          </Typography.Paragraph>

          {rawView ? (
            <CodeEditor
              title="Config"
              value={rawText}
              onChange={(val) => setRawText(val ?? '')}
              language="json"
              minimal
            />
          ) : (
            // One shared Form across child pages. Children render Form.Item and get context automatically.
            <Form layout="vertical" form={form} className="space-y-6">
              <Outlet context={outletCtx} />
            </Form>
          )}

          {/* Buttons removed; child pages render actions using useAssignmentConfig() */}
        </div>
      </div>

      {/* Mobile: just the children/form; child pages show their own actions */}
      <div className="block sm:hidden">
        <Form layout="vertical" form={form}>
          <Outlet context={outletCtx} />
        </Form>
      </div>
    </div>
  );
};

export default ConfigLayout;
