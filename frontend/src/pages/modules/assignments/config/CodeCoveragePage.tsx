import { useEffect, useCallback } from 'react';
import { Form, InputNumber, Typography, Space, Select } from 'antd';
import SettingsGroup from '@/components/SettingsGroup';
import { useViewSlot } from '@/context/ViewSlotContext';
import { useAssignment } from '@/context/AssignmentContext';
import { message } from '@/utils/message';
import type { AssignmentConfig } from '@/types/modules/assignments/config';
import AssignmentConfigActions from '@/components/assignments/AssignmentConfigActions';
import Tip from '@/components/common/Tip';
import { useBreadcrumbContext } from '@/context/BreadcrumbContext';
import useConfigBackTo from '@/hooks/useConfigBackTo';

type FormShape = {
  code_coverage_weight: number;
  code_coverage_whitelist: string[];
};

export default function CodeCoveragePage() {
  useConfigBackTo();
  const { setValue } = useViewSlot();
  const { config, updateConfig, assignment } = useAssignment();
  const { setBreadcrumbLabel } = useBreadcrumbContext();

  const [form] = Form.useForm<FormShape>();

  useEffect(() => {
    setBreadcrumbLabel(
      `modules/${assignment.module_id}/assignments/${assignment.id}/config/code-coverage`,
      'Code Coverage',
    );
  }, []);

  useEffect(() => {
    setValue(
      <Space align="center" size={6} className="flex-wrap">
        <Typography.Text className="text-base font-medium text-gray-900 dark:text-gray-100 truncate">
          Code Coverage
        </Typography.Text>
        <Tip iconOnly newTab to="/help/assignments/coverage#overview" text="Coverage help" />
      </Space>,
    );
  }, [setValue]);

  useEffect(() => {
    const cc = config?.code_coverage;
    if (!cc) return;

    form.setFieldsValue({
      // UI stays in percent; backend expects same number you set here.
      code_coverage_weight:
        typeof cc.code_coverage_weight === 'number' ? cc.code_coverage_weight : 10,
      // Whitelist is a simple list of file names to count toward coverage.
      code_coverage_whitelist: Array.isArray(cc.whitelist) ? cc.whitelist : [],
    });
  }, [config?.code_coverage, form]);

  const onSave = useCallback(async () => {
    if (!config) return message.error('No configuration loaded yet.');
    const values = await form.validateFields();
    try {
      const patch: Partial<AssignmentConfig> = {
        code_coverage: {
          code_coverage_weight: values.code_coverage_weight,
          whitelist: (values.code_coverage_whitelist || [])
            .map((s) => String(s).trim())
            .filter(Boolean),
        },
      };
      await updateConfig(patch);
      message.success('Coverage settings saved');
    } catch (e: any) {
      message.error(e?.message ?? 'Failed to save coverage settings');
    }
  }, [config, form, updateConfig]);

  const disabled = !config;

  return (
    <SettingsGroup
      title="Code Coverage"
      description="Choose how much of the mark comes from coverage, and optionally list the file names that should count toward coverage."
    >
      <Form layout="vertical" form={form} className="space-y-6" disabled={disabled}>
        <Form.Item
          name="code_coverage_weight"
          label="Coverage Weight (%)"
          className="w-full max-w-xs"
          rules={[
            { required: true, message: 'Enter a percentage' },
            { type: 'number', min: 0, max: 95, message: 'Recommended 0–95' },
          ]}
          extra="Example: 10 means ~10% of the total mark comes from coverage."
        >
          <InputNumber className="w-full" min={0} max={100} precision={2} step={0.25} />
        </Form.Item>

        <Form.Item
          name="code_coverage_whitelist"
          label="Coverage File Whitelist"
          tooltip="Optional. If provided, only these file names will be included when calculating code coverage."
          extra="Enter plain file names (no globs/paths). Examples: main.cpp, utils.c, MyClass.java"
        >
          <Select
            mode="tags"
            tokenSeparators={[',', ' ']}
            placeholder="e.g., main.cpp, utils.c, MyClass.java"
            style={{ maxWidth: 375 }}
          />
        </Form.Item>

        <div className="pt-2">
          <AssignmentConfigActions
            primaryText="Save Coverage Settings"
            onPrimary={onSave}
            disabled={disabled}
          />
        </div>
      </Form>
    </SettingsGroup>
  );
}
