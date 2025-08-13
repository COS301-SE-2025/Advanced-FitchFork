import { Form, Input, Select } from 'antd';
import SettingsGroup from '@/components/SettingsGroup';
import {
  FEEDBACK_SCHEME_OPTIONS,
  MARKING_SCHEME_OPTIONS,
} from '@/types/modules/assignments/config';

export default function MarkingPage() {
  return (
    <SettingsGroup
      title="Marking & Feedback"
      description="Determine how submissions are evaluated and feedback is generated."
    >
      <Form.Item name={['marking', 'marking_scheme']} label="Marking Scheme">
        <Select className="!w-60" options={MARKING_SCHEME_OPTIONS} />
      </Form.Item>

      <Form.Item name={['marking', 'feedback_scheme']} label="Feedback Scheme">
        <Select className="!w-60" options={FEEDBACK_SCHEME_OPTIONS} />
      </Form.Item>

      <Form.Item name={['marking', 'deliminator']} label="Delimiter String">
        <Input className="!w-60" addonBefore="Delimiter" />
      </Form.Item>
    </SettingsGroup>
  );
}
