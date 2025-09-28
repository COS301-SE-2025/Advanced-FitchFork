import { useEffect, useCallback } from 'react';
import { Form, Input, Select, Typography, Switch, InputNumber, Space } from 'antd';
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
import Tip from '@/components/common/Tip';
import useConfigBackTo from '@/hooks/useConfigBackTo';

type FormShape = AssignmentMarkingConfig;

export default function MarkingPage() {
  useConfigBackTo();
  const { setValue } = useViewSlot();
  const { config, updateConfig } = useAssignment();
  const [form] = Form.useForm<FormShape>();

  useEffect(() => {
    setValue(
      <Space align="center" size={6} className="flex-wrap">
        <Typography.Text className="text-base font-medium text-gray-900 dark:text-gray-100 truncate">
          Marking Configuration
        </Typography.Text>
        <Tip iconOnly newTab to="/help/assignments/config/marking#what" text="Marking help" />
      </Space>,
    );
  }, [setValue]);

  // Seed form whenever config.marking changes (+ sensible defaults)
  useEffect(() => {
    if (!config?.marking) return;
    form.setFieldsValue({
      ...config.marking,
      dissalowed_code: config.marking.dissalowed_code ?? [],
      late: {
        allow_late_submissions: config.marking.late?.allow_late_submissions ?? false,
        late_window_minutes: config.marking.late?.late_window_minutes ?? 0,
        late_max_percent: config.marking.late?.late_max_percent ?? 100,
      },
    });
  }, [config?.marking, form]);

  const onSave = useCallback(async () => {
    if (!config) return message.error('No configuration loaded yet.');
    const values = await form.validateFields();

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
        dissalowed_code: values.dissalowed_code ?? [],
        late: {
          allow_late_submissions: values.late.allow_late_submissions,
          late_window_minutes: values.late.late_window_minutes,
          late_max_percent: values.late.late_max_percent,
        },
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
    <Form layout="vertical" form={form} className="space-y-6" disabled={disabled}>
      {/* ===== Group: Marking & Feedback ===== */}
      <SettingsGroup
        title="Marking & Feedback"
        description="Determine how submissions are evaluated and how feedback is generated."
      >
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

        {/* Practice submissions (students only) */}
        <Form.Item
          name="allow_practice_submissions"
          label="Allow Practice Submissions"
          valuePropName="checked"
          className={fieldWidth}
          tooltip="When ON, students may upload practice submissions. Staff are always allowed regardless of this setting."
        >
          <Switch />
        </Form.Item>

        {/* Attempts */}
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
          <Input className="w-full" placeholder="e.g., ###" />
        </Form.Item>

        {/* Disallowed code substrings */}
        <Form.Item
          name="dissalowed_code"
          label="Disallowed Code Substrings"
          className="w-full max-w-2xl"
          tooltip="Any file inside a submitted ZIP containing one of these substrings will be flagged."
          extra="Tip: press Enter or comma after each token."
          rules={[{ type: 'array' }]}
        >
          <Select
            mode="tags"
            tokenSeparators={[',', '\n']}
            placeholder="e.g., import forbidden_code, system(, eval("
            style={{ maxWidth: 375 }}
          />
        </Form.Item>
      </SettingsGroup>

      {/* ===== Group: Late Submissions ===== */}
      <SettingsGroup
        title="Late Submissions"
        description="Accept late submissions within a grace window and optionally cap the awarded mark."
      >
        <Form.Item
          name={['late', 'allow_late_submissions']}
          label="Allow Late Submissions"
          valuePropName="checked"
          className={fieldWidth}
          tooltip="If OFF, late uploads are rejected. If ON, uploads after due date are accepted within the grace window below."
          rules={[{ type: 'boolean' }]}
        >
          <Switch />
        </Form.Item>

        <Form.Item
          noStyle
          shouldUpdate={(prev, cur) =>
            prev?.late?.allow_late_submissions !== cur?.late?.allow_late_submissions
          }
        >
          {({ getFieldValue }) =>
            getFieldValue(['late', 'allow_late_submissions']) ? (
              <>
                <Form.Item
                  name={['late', 'late_window_minutes']}
                  label="Late Window (minutes)"
                  className={fieldWidth}
                  rules={[
                    { required: true, message: 'Enter a grace window' },
                    { type: 'number', min: 0, message: 'Must be ≥ 0' },
                  ]}
                  extra="Submissions after the due date are accepted up to this many minutes late."
                >
                  <InputNumber className="w-full" min={0} precision={0} />
                </Form.Item>

                <Form.Item
                  name={['late', 'late_max_percent']}
                  label="Cap Awarded Mark (%)"
                  className={fieldWidth}
                  rules={[
                    { required: true, message: 'Enter a cap percentage' },
                    { type: 'number', min: 0, max: 100, message: 'Must be between 0 and 100' },
                  ]}
                  extra="If the earned mark exceeds this percentage of the total, it will be capped."
                >
                  <InputNumber className="w-full" min={0} max={100} precision={0} />
                </Form.Item>
              </>
            ) : null
          }
        </Form.Item>
      </SettingsGroup>

      <div className="pt-2">
        <AssignmentConfigActions
          primaryText="Save Marking Config"
          onPrimary={onSave}
          disabled={disabled}
        />
      </div>
    </Form>
  );
}
