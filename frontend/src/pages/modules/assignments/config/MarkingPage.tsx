import { useEffect, useCallback } from 'react';
import { Form, Input, Select, Typography } from 'antd';
import SettingsGroup from '@/components/SettingsGroup';
import {
  FEEDBACK_SCHEME_OPTIONS,
  MARKING_SCHEME_OPTIONS,
  type AssignmentMarkingConfig,
} from '@/types/modules/assignments/config';
import { useViewSlot } from '@/context/ViewSlotContext';
import { useAssignment } from '@/context/AssignmentContext';
import { message } from '@/utils/message';
import AssignmentConfigActions from '@/components/assignments/AssignmentConfigActions';

export default function MarkingPage() {
  const { setValue } = useViewSlot();
  const { config, updateConfig } = useAssignment();
  const [form] = Form.useForm<AssignmentMarkingConfig>();

  useEffect(() => {
    setValue(
      <Typography.Text className="text-base font-medium text-gray-900 dark:text-gray-100 truncate">
        Marking Configuration
      </Typography.Text>,
    );
  }, [setValue]);

  // Seed form whenever config.marking changes
  useEffect(() => {
    if (!config?.marking) return;
    form.setFieldsValue(config.marking);
  }, [config?.marking, form]);

  const onSave = useCallback(async () => {
    if (!config) return message.error('No configuration loaded yet.');
    const values = await form.validateFields(); // AssignmentMarkingConfig
    try {
      await updateConfig({ marking: values });
      message.success('Marking config saved');
    } catch (e: any) {
      message.error(e?.message ?? 'Failed to save marking config');
    }
  }, [config, form, updateConfig]);

  const fieldWidth = 'w-full max-w-xs';
  const textFieldWidth = 'w-full max-w-sm';
  const disabled = !config;

  return (
    <SettingsGroup
      title="Marking & Feedback"
      description="Determine how submissions are evaluated and how feedback is generated."
    >
      <Form layout="vertical" form={form} className="space-y-6" disabled={disabled}>
        <Form.Item
          name="marking_scheme"
          label="Marking Scheme"
          className={fieldWidth}
          rules={[{ required: true, message: 'Select a marking scheme' }]}
        >
          <Select className="w-full" options={MARKING_SCHEME_OPTIONS} />
        </Form.Item>

        <Form.Item
          name="feedback_scheme"
          label="Feedback Scheme"
          className={fieldWidth}
          rules={[{ required: true, message: 'Select a feedback scheme' }]}
        >
          <Select className="w-full" options={FEEDBACK_SCHEME_OPTIONS} />
        </Form.Item>

        <Form.Item
          name="deliminator" // matches backend field name intentionally
          label="Delimiter String"
          className={textFieldWidth}
          rules={[{ required: true, message: 'Enter a delimiter string' }]}
          extra="Used to split output sections when parsing results."
        >
          <Input className="w-full" placeholder="e.g., &-=-&" />
        </Form.Item>

        <div className="pt-2">
          <AssignmentConfigActions
            primaryText="Save Marking Config"
            onPrimary={onSave}
            disabled={disabled}
          />
        </div>
      </Form>
    </SettingsGroup>
  );
}
