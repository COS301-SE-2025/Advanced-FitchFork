import { Form, InputNumber, Typography } from 'antd';
import SettingsGroup from '@/components/SettingsGroup';
import { ConfigActions } from '@/context/AssignmentConfigContext';
import { useViewSlot } from '@/context/ViewSlotContext';
import { useEffect } from 'react';

const toMB = (bytes: number) => Math.round(bytes / (1024 * 1024));
const toBytes = (mb: number) => Math.round(mb * 1024 * 1024);

export default function ExecutionPage() {
  const { setValue } = useViewSlot();

  useEffect(() => {
    setValue(
      <Typography.Text className="text-base font-medium text-gray-900 dark:text-gray-100 truncate">
        Execution Configuration
      </Typography.Text>,
    );
  }, []);

  // tweak here if you want a different cap (e.g. 'max-w-sm' ~ 24rem)
  const fieldWidth = 'w-full max-w-xs';

  return (
    <div>
      <SettingsGroup
        title="Execution Limits"
        description="Control how student code is run and isolated."
      >
        <Form.Item name={['execution', 'timeout_secs']} label="Timeout" className={fieldWidth}>
          <InputNumber min={1} className="w-full" addonAfter="sec" />
        </Form.Item>

        <Form.Item
          name={['execution', 'max_memory']}
          label="Max Memory"
          getValueProps={(value) => ({ value: toMB(value) })}
          normalize={(value) => toBytes(value)}
          className={fieldWidth}
        >
          <InputNumber min={1} className="w-full" addonAfter="MB" />
        </Form.Item>

        <Form.Item name={['execution', 'max_cpus']} label="Max CPUs" className={fieldWidth}>
          <InputNumber min={1} step={1} precision={0} className="w-full" addonAfter="cores" />
        </Form.Item>

        <Form.Item
          name={['execution', 'max_uncompressed_size']}
          label="Max Uncompressed Size"
          getValueProps={(value) => ({ value: toMB(value) })}
          normalize={(value) => toBytes(value)}
          className={fieldWidth}
        >
          <InputNumber min={1} className="w-full" addonAfter="MB" />
        </Form.Item>

        <Form.Item
          name={['execution', 'max_processes']}
          label="Max Processes"
          className={fieldWidth}
        >
          <InputNumber min={1} className="w-full" />
        </Form.Item>

        <ConfigActions saveLabel="Save Execution Config" />
      </SettingsGroup>
    </div>
  );
}
