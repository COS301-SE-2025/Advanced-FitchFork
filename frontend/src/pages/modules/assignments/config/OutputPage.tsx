import { useEffect, useCallback } from 'react';
import { Form, Switch, Typography } from 'antd';
import SettingsGroup from '@/components/SettingsGroup';
import { useViewSlot } from '@/context/ViewSlotContext';
import { useAssignment } from '@/context/AssignmentContext';
import { message } from '@/utils/message';
import type { AssignmentOutputConfig } from '@/types/modules/assignments/config';
import AssignmentConfigActions from '@/components/assignments/AssignmentConfigActions';

export default function OutputPage() {
  const { setValue } = useViewSlot();
  const { config, updateConfig } = useAssignment();
  const [form] = Form.useForm<AssignmentOutputConfig>();

  useEffect(() => {
    setValue(
      <Typography.Text className="text-base font-medium text-gray-900 dark:text-gray-100 truncate">
        Output Settings
      </Typography.Text>,
    );
  }, [setValue]);

  // Seed form whenever config.output changes
  useEffect(() => {
    if (!config?.output) return;
    form.setFieldsValue(config.output);
  }, [config?.output, form]);

  const onSave = useCallback(async () => {
    if (!config) return message.error('No configuration loaded yet.');
    const values = await form.validateFields(); // AssignmentOutputConfig
    try {
      await updateConfig({ output: values });
      message.success('Output settings saved');
    } catch (e: any) {
      message.error(e?.message ?? 'Failed to save output settings');
    }
  }, [config, form, updateConfig]);

  const disabled = !config;
  const fieldWidth = 'w-full max-w-xs';

  return (
    <div>
      <SettingsGroup
        title="Output Capture"
        description="Choose which streams and metadata to capture from student runs."
      >
        <Form layout="vertical" form={form} disabled={disabled} className="space-y-6">
          <Form.Item
            name="stdout"
            label="Capture STDOUT"
            valuePropName="checked"
            extra="Standard output (e.g., printed results)."
            className={fieldWidth}
          >
            <Switch />
          </Form.Item>

          <Form.Item
            name="stderr"
            label="Capture STDERR"
            valuePropName="checked"
            extra="Error output (e.g., stack traces and diagnostics)."
            className={fieldWidth}
          >
            <Switch />
          </Form.Item>

          <Form.Item
            name="retcode"
            label="Include Return Code"
            valuePropName="checked"
            extra="Record the process exit status (0 = success, non-zero = failure)."
            className={fieldWidth}
          >
            <Switch />
          </Form.Item>

          <div className="pt-2">
            <AssignmentConfigActions
              primaryText="Save Output Settings"
              onPrimary={onSave}
              disabled={disabled}
            />
          </div>
        </Form>
      </SettingsGroup>
    </div>
  );
}
