import { Form, Select, Typography } from 'antd';
import SettingsGroup from '@/components/SettingsGroup';
import { ConfigActions } from '@/context/AssignmentConfigContext';
import { useViewSlot } from '@/context/ViewSlotContext';
import { useEffect } from 'react';
import { LANGUAGE_OPTIONS, SUBMISSION_MODE_OPTIONS } from '@/types/modules/assignments/config';

export default function AssignmentPage() {
  const { setValue } = useViewSlot();

  useEffect(() => {
    setValue(
      <Typography.Text className="text-base font-medium text-gray-900 dark:text-gray-100 truncate">
        Assignment Configuration
      </Typography.Text>,
    );
  }, []);

  const fieldWidth = 'w-full max-w-xs'; // widen to max-w-sm/md if needed

  return (
    <SettingsGroup
      title="Assignment Setup"
      description="General assignment configuration: language and how submissions are produced."
    >
      <Form.Item
        name={['project', 'language']}
        label="Project Language"
        tooltip="Language used for compiling and running student submissions."
        className={fieldWidth}
      >
        <Select className="w-full" options={LANGUAGE_OPTIONS} />
      </Form.Item>

      <Form.Item
        name={['project', 'submission_mode']}
        label="Submission Mode"
        tooltip="How submissions are generated: manual (lecturer-provided), gatlam (GA/TLAM-produced), rng (randomized), or codecoverage (coverage-driven)."
        className={fieldWidth}
      >
        <Select className="w-full" options={SUBMISSION_MODE_OPTIONS} />
      </Form.Item>

      <ConfigActions saveLabel="Save Assignment Config" />
    </SettingsGroup>
  );
}
