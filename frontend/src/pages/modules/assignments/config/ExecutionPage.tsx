import { useEffect, useCallback } from 'react';
import { Form, InputNumber, Typography, Space } from 'antd';
import SettingsGroup from '@/components/SettingsGroup';
import { useViewSlot } from '@/context/ViewSlotContext';
import { useAssignment } from '@/context/AssignmentContext';
import { message } from '@/utils/message';
import type { AssignmentExecutionConfig } from '@/types/modules/assignments/config';
import AssignmentConfigActions from '@/components/assignments/AssignmentConfigActions';
import Tip from '@/components/common/Tip';
import useConfigBackTo from '@/hooks/useConfigBackTo';

const toMB = (bytes: number) => Math.round(bytes / (1024 * 1024));
const toBytes = (mb: number) => Math.round(mb * 1024 * 1024);

export default function ExecutionPage() {
  useConfigBackTo();
  const { setValue } = useViewSlot();
  const { config, updateConfig } = useAssignment();
  const [form] = Form.useForm<AssignmentExecutionConfig>();

  useEffect(() => {
    setValue(
      <Space align="center" size={6} className="flex-wrap">
        <Typography.Text className="text-base font-medium text-gray-900 dark:text-gray-100 truncate">
          Execution Configuration
        </Typography.Text>
        <Tip iconOnly newTab to="/help/assignments/config/execution#what" text="Execution help" />
      </Space>,
    );
  }, [setValue]);

  // Seed form whenever config.execution changes
  useEffect(() => {
    const exec = config?.execution;
    if (!exec) return;
    form.setFieldsValue({
      ...exec,
      max_memory: toMB(exec.max_memory),
      max_uncompressed_size: toMB(exec.max_uncompressed_size),
    });
  }, [config?.execution, form]);

  const onSave = useCallback(async () => {
    if (!config) return message.error('No configuration loaded yet.');
    const values = await form.validateFields();
    try {
      await updateConfig({
        execution: {
          ...values,
          max_memory: toBytes(values.max_memory),
          max_uncompressed_size: toBytes(values.max_uncompressed_size),
        },
      });
      message.success('Execution config saved');
    } catch (e: any) {
      message.error(e?.message ?? 'Failed to save execution config');
    }
  }, [config, form, updateConfig]);

  const fieldWidth = 'w-full max-w-xs';
  const disabled = !config;

  return (
    <div>
      <SettingsGroup
        title="Execution Limits"
        description="Control how student code is run and isolated."
      >
        <Form layout="vertical" form={form} className="space-y-6" disabled={disabled}>
          <Form.Item
            name="timeout_secs"
            label="Timeout"
            className={fieldWidth}
            rules={[{ required: true }]}
          >
            <InputNumber min={1} className="w-full" addonAfter="sec" />
          </Form.Item>

          <Form.Item
            name="max_memory"
            label="Max Memory"
            className={fieldWidth}
            rules={[{ required: true }]}
          >
            <InputNumber min={1} className="w-full" addonAfter="MB" />
          </Form.Item>

          <Form.Item
            name="max_cpus"
            label="Max CPUs"
            className={fieldWidth}
            rules={[{ required: true }]}
          >
            <InputNumber min={1} step={1} precision={0} className="w-full" addonAfter="cores" />
          </Form.Item>

          <Form.Item
            name="max_uncompressed_size"
            label="Max Uncompressed Size"
            className={fieldWidth}
            rules={[{ required: true }]}
          >
            <InputNumber min={1} className="w-full" addonAfter="MB" />
          </Form.Item>

          <Form.Item
            name="max_processes"
            label="Max Processes"
            className={fieldWidth}
            rules={[{ required: true }]}
          >
            <InputNumber min={1} className="w-full" />
          </Form.Item>

          <div className="pt-2">
            <AssignmentConfigActions
              primaryText="Save Execution Config"
              onPrimary={onSave}
              disabled={disabled}
            />
          </div>
        </Form>
      </SettingsGroup>
    </div>
  );
}
