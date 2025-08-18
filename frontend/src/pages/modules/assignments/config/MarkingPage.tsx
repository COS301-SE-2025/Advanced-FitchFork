import { Form, Input, Select, Typography } from 'antd';
import SettingsGroup from '@/components/SettingsGroup';
import {
  FEEDBACK_SCHEME_OPTIONS,
  MARKING_SCHEME_OPTIONS,
} from '@/types/modules/assignments/config';
import { ConfigActions } from '@/context/AssignmentConfigContext';
import { useViewSlot } from '@/context/ViewSlotContext';
import { useEffect } from 'react';

export default function MarkingPage() {
  const { setValue } = useViewSlot();

  useEffect(() => {
    setValue(
      <Typography.Text className="text-base font-medium text-gray-900 dark:text-gray-100 truncate">
        Marking Configuration
      </Typography.Text>,
    );
  }, []);

  const fieldWidth = 'w-full max-w-xs'; // ~320px
  const textFieldWidth = 'w-full max-w-sm'; // ~384px (a touch wider for the delimiter)

  return (
    <SettingsGroup
      title="Marking & Feedback"
      description="Determine how submissions are evaluated and feedback is generated."
    >
      <Form.Item name={['marking', 'marking_scheme']} label="Marking Scheme" className={fieldWidth}>
        <Select className="w-full" options={MARKING_SCHEME_OPTIONS} />
      </Form.Item>

      <Form.Item
        name={['marking', 'feedback_scheme']}
        label="Feedback Scheme"
        className={fieldWidth}
      >
        <Select className="w-full" options={FEEDBACK_SCHEME_OPTIONS} />
      </Form.Item>

      <Form.Item
        name={['marking', 'deliminator']}
        label="Delimiter String"
        className={textFieldWidth}
      >
        <Input className="w-full" addonBefore="Delimiter" />
      </Form.Item>

      <ConfigActions saveLabel="Save Marking Config" />
    </SettingsGroup>
  );
}
