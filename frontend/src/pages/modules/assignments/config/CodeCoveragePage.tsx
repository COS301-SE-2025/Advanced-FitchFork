import { useEffect, useCallback } from 'react';
import { Form, InputNumber, Typography, Space } from 'antd';
import SettingsGroup from '@/components/SettingsGroup';
import { useViewSlot } from '@/context/ViewSlotContext';
import { useAssignment } from '@/context/AssignmentContext';
import { message } from '@/utils/message';
import type { AssignmentConfig } from '@/types/modules/assignments/config';
import AssignmentConfigActions from '@/components/assignments/AssignmentConfigActions';
import Tip from '@/components/common/Tip';

export default function CodeCoveragePage() {
  const { setValue } = useViewSlot();
  const { config, updateConfig } = useAssignment();
  const [form] = Form.useForm<{ code_coverage_required: number }>();

  useEffect(() => {
    setValue(
      <Space align="center" size={6} className="flex-wrap">
        <Typography.Text className="text-base font-medium text-gray-900 dark:text-gray-100 truncate">
          Code Coverage
        </Typography.Text>
        <Tip
          iconOnly
          newTab
          to="/help/assignments/coverage#overview"
          text="Coverage help"
        />
      </Space>,
    );
  }, [setValue]);

  useEffect(() => {
    const cc = config?.code_coverage;
    if (!cc) return;
    form.setFieldsValue({ code_coverage_required: cc.code_coverage_required ?? 80 });
  }, [config?.code_coverage, form]);

  const onSave = useCallback(async () => {
    if (!config) return message.error('No configuration loaded yet.');
    const values = await form.validateFields();
    try {
      const patch: Partial<AssignmentConfig> = {
        code_coverage: { code_coverage_required: values.code_coverage_required },
      };
      await updateConfig(patch);
      message.success('Coverage target saved');
    } catch (e: any) {
      message.error(e?.message ?? 'Failed to save coverage target');
    }
  }, [config, form, updateConfig]);

  const disabled = !config;

  return (
    <SettingsGroup
      title="Code Coverage"
      description="Set a target percentage for assignments that measure code coverage."
    >
      <Form layout="vertical" form={form} className="space-y-6" disabled={disabled}>
        <Form.Item
          name="code_coverage_required"
          label="Required Coverage (%)"
          className="w-full max-w-xs"
          rules={[
            { required: true, message: 'Enter a percentage' },
            { type: 'number', min: 0, max: 100, message: 'Must be 0–100' },
          ]}
          extra="Used in coverage workflows and reports."
        >
          <InputNumber className="w-full" min={0} max={100} precision={0} />
        </Form.Item>

        <div className="pt-2">
          <AssignmentConfigActions
            primaryText="Save Coverage Target"
            onPrimary={onSave}
            disabled={disabled}
          />
        </div>
      </Form>
    </SettingsGroup>
  );
}
