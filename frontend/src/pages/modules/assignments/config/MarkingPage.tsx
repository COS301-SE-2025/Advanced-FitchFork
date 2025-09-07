import { useEffect, useCallback } from 'react';
import { Form, Input, Select, Typography, Switch, InputNumber } from 'antd';
import SettingsGroup from '@/components/SettingsGroup';
import {
  FEEDBACK_SCHEME_OPTIONS,
  MARKING_SCHEME_OPTIONS,
  GRADING_POLICY_OPTIONS,
  type AssignmentMarkingConfig,
} from '@/types/modules/assignments/config';
import { useViewSlot } from '@/context/ViewSlotContext';
import { useAssignment } from '@/context/AssignmentContext';
import { message } from '@/utils/message';
import AssignmentConfigActions from '@/components/assignments/AssignmentConfigActions';

type FormShape = AssignmentMarkingConfig;

export default function MarkingPage() {
  const { setValue } = useViewSlot();
  const { config, updateConfig } = useAssignment();
  const [form] = Form.useForm<FormShape>();

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
    const values = await form.validateFields(); // FormShape

    try {
      const toSave: AssignmentMarkingConfig = {
        marking_scheme: values.marking_scheme,
        feedback_scheme: values.feedback_scheme,
        grading_policy: values.grading_policy,
        deliminator: values.deliminator,
        max_attempts: values.max_attempts,
        limit_attempts: values.limit_attempts,
        pass_mark: values.pass_mark,
        allow_practice_submissions: values.allow_practice_submissions,
      };
      await updateConfig({ marking: toSave });
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
          name="grading_policy"
          label="Grading Policy"
          className={fieldWidth}
          rules={[{ required: true, message: 'Select a grading policy' }]}
        >
          <Select className="w-full" options={GRADING_POLICY_OPTIONS} />
        </Form.Item>

        {/* ---- Practice submissions (students only) ---- */}
        <Form.Item
          name="allow_practice_submissions"
          label="Allow Practice Submissions"
          valuePropName="checked"
          className={fieldWidth}
          tooltip="When ON, students may upload practice submissions. Staff are always allowed regardless of this setting."
        >
          <Switch />
        </Form.Item>

        {/* ---- Attempts ---- */}
        <div className="grid gap-2">
          <Form.Item
            name="limit_attempts"
            label="Limit Attempts"
            valuePropName="checked"
            className={fieldWidth}
            tooltip="If OFF: students have unlimited non-practice attempts. (Staff are never limited.)"
          >
            <Switch />
          </Form.Item>

          <Form.Item
            noStyle
            shouldUpdate={(prev, cur) => prev.limit_attempts !== cur.limit_attempts}
          >
            {({ getFieldValue }) =>
              getFieldValue('limit_attempts') ? (
                <Form.Item
                  name="max_attempts"
                  label="Max Attempts"
                  className={fieldWidth}
                  rules={[
                    { required: true, message: 'Enter a max attempts value' },
                    { type: 'number', min: 1, message: 'Must be at least 1' },
                  ]}
                  extra="Counts only non-practice, non-ignored submissions (students only)."
                >
                  <InputNumber className="w-full" min={1} precision={0} />
                </Form.Item>
              ) : null
            }
          </Form.Item>
        </div>

        <Form.Item
          name="pass_mark"
          label="Pass Mark (%)"
          className={fieldWidth}
          rules={[
            { required: true, message: 'Enter a pass mark' },
            { type: 'number', min: 0, max: 100, message: 'Must be between 0 and 100' },
          ]}
          extra="Minimum percentage required to pass."
        >
          <InputNumber className="w-full" min={0} max={100} precision={0} />
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
