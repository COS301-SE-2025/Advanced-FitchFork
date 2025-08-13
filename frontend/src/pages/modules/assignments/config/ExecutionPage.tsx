import { Form, InputNumber } from 'antd';
import SettingsGroup from '@/components/SettingsGroup';

const toMB = (bytes: number) => Math.round(bytes / (1024 * 1024));
const toBytes = (mb: number) => Math.round(mb * 1024 * 1024);

export default function ExecutionPage() {
  return (
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
  );
}
