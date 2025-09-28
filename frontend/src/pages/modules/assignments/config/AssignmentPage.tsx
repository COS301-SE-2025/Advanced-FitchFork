import { useEffect } from 'react';
import { Form, Select, Typography, Space } from 'antd';
import SettingsGroup from '@/components/SettingsGroup';
import { useViewSlot } from '@/context/ViewSlotContext';
import { useAssignment } from '@/context/AssignmentContext';
import { message } from '@/utils/message';

import {
  LANGUAGE_OPTIONS,
  SUBMISSION_MODE_OPTIONS,
  type AssignmentProjectConfig,
} from '@/types/modules/assignments/config';

import AssignmentConfigActions from '@/components/assignments/AssignmentConfigActions';
import Tip from '@/components/common/Tip';
import useConfigBackTo from '@/hooks/useConfigBackTo';

export default function AssignmentPage() {
  useConfigBackTo();
  const { setValue } = useViewSlot();
  const { config, updateConfig } = useAssignment();
  const [form] = Form.useForm<AssignmentProjectConfig>(); // ← use existing type

  useEffect(() => {
    setValue(
      <Space align="center" size={6} className="flex-wrap">
        <Typography.Text className="text-base font-medium text-gray-900 dark:text-gray-100 truncate">
          Assignment Configuration
        </Typography.Text>
        <Tip
          iconOnly
          newTab
          to="/help/assignments/config/project#what"
          text="Project config help"
        />
      </Space>,
    );
  }, [setValue]);

  // seed form whenever context config changes
  useEffect(() => {
    if (!config?.project) return;
    form.setFieldsValue({
      language: config.project.language,
      submission_mode: config.project.submission_mode,
    });
  }, [config?.project, form]);

  const onSave = async () => {
    if (!config) {
      message.error('No configuration loaded yet.');
      return;
    }

    const values = await form.validateFields(); // AssignmentProjectConfig
    try {
      await updateConfig({ project: values }); // provider merges & POSTs full config
      message.success('Assignment config saved');
    } catch (e: any) {
      message.error(e?.message ?? 'Failed to save assignment config');
    }
  };

  const fieldWidth = 'w-full max-w-xs';
  const disabled = !config;

  return (
    <SettingsGroup
      title="Assignment Setup"
      description="General assignment configuration: language and how submissions are produced."
    >
      <Form layout="vertical" form={form} className="space-y-6" disabled={disabled}>
        <Form.Item
          name="language"
          label="Project Language"
          tooltip="Language used for compiling and running student submissions."
          className={fieldWidth}
          rules={[{ required: true, message: 'Select a language' }]}
        >
          <Select className="w-full" options={LANGUAGE_OPTIONS} />
        </Form.Item>

        <Form.Item
          name="submission_mode"
          label="Submission Mode"
          tooltip="How submissions are generated: manual (lecturer-provided), gatlam (GA/TLAM-produced), rng (randomized), or codecoverage (coverage-driven)."
          className={fieldWidth}
          rules={[{ required: true, message: 'Select a submission mode' }]}
        >
          <Select className="w-full" options={SUBMISSION_MODE_OPTIONS} />
        </Form.Item>

        <div className="pt-2">
          <AssignmentConfigActions
            primaryText="Save Assignment Config"
            onPrimary={onSave}
            disabled={disabled}
            // Reset handled inside the shared component (centered Modal.confirm)
          />
        </div>
      </Form>
    </SettingsGroup>
  );
}
