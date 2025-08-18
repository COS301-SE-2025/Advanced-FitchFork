import { Form, Switch, Typography } from 'antd';
import SettingsGroup from '@/components/SettingsGroup';
import { ConfigActions } from '@/context/AssignmentConfigContext';
import { useViewSlot } from '@/context/ViewSlotContext';
import { useEffect } from 'react';

export default function OutputPage() {
  const { setValue } = useViewSlot();

  useEffect(() => {
    setValue(
      <Typography.Text className="text-base font-medium text-gray-900 dark:text-gray-100 truncate">
        Output Settings
      </Typography.Text>,
    );
  }, []);

  return (
    <div>
      <SettingsGroup
        title="Output Capture"
        description="Choose which streams and metadata to capture from student runs."
      >
        <Form.Item
          name={['output', 'stdout']}
          label="Capture STDOUT"
          valuePropName="checked"
          extra="Standard output (e.g., printed results)."
        >
          <Switch />
        </Form.Item>

        <Form.Item
          name={['output', 'stderr']}
          label="Capture STDERR"
          valuePropName="checked"
          extra="Error output (e.g., stack traces and diagnostics)."
        >
          <Switch />
        </Form.Item>

        <Form.Item
          name={['output', 'retcode']}
          label="Include Return Code"
          valuePropName="checked"
          extra="Record the process exit status (0 = success, non-zero = failure)."
        >
          <Switch />
        </Form.Item>

        <ConfigActions saveLabel="Save Output Settings" />
      </SettingsGroup>
    </div>
  );
}
