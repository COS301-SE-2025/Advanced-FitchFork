import { useEffect, useCallback, useMemo } from 'react';
import { Form, Input, Select, Typography, Switch, InputNumber, Button, Space } from 'antd';
import SettingsGroup from '@/components/SettingsGroup';
import { useViewSlot } from '@/context/ViewSlotContext';
import { useAssignment } from '@/context/AssignmentContext';
import { message } from '@/utils/message';
import AssignmentConfigActions from '@/components/assignments/AssignmentConfigActions';
import type { AssignmentSecurityConfig } from '@/types/modules/assignments/config';

type FormShape = {
  password_enabled: boolean;
  password_pin?: string | null;
  cookie_ttl_minutes: number;
  bind_cookie_to_user: boolean;
  allowed_cidrs: string[];
};

/* ─── Constants (no magic numbers) ───────────────────────────── */
const PIN_LENGTH = 6 as const;
const MIN_COOKIE_TTL_MINUTES = 1 as const;

const cidrHelp =
  'Optional allowlist of CIDRs (IPv4/IPv6). Leave empty to allow all. Example IPv4: 10.0.0.0/24, 196.21.0.0/16';

const ipv4CidrRe = /^(\d{1,3}\.){3}\d{1,3}\/\d{1,2}$/;
// very light IPv6 CIDR check (format sanity only)
const ipv6CidrRe = /^[0-9a-f:]+\/\d{1,3}$/i;

// Numeric PIN (1–9 only)
const PIN_CHARS = '123456789';
const generatePin = (len = PIN_LENGTH) =>
  Array.from({ length: len }, () => PIN_CHARS[Math.floor(Math.random() * PIN_CHARS.length)]).join(
    '',
  );

export default function SecurityPage() {
  const { setValue } = useViewSlot();
  const { config, updateConfig } = useAssignment();
  const [form] = Form.useForm<FormShape>();

  useEffect(() => {
    setValue(
      <Typography.Text className="text-base font-medium text-gray-900 dark:text-gray-100 truncate">
        Security
      </Typography.Text>,
    );
  }, [setValue]);

  // Seed form whenever config.security changes
  useEffect(() => {
    if (!config?.security) return;
    form.setFieldsValue({
      password_enabled: config.security.password_enabled,
      password_pin: config.security.password_pin ?? '',
      cookie_ttl_minutes: config.security.cookie_ttl_minutes,
      bind_cookie_to_user: config.security.bind_cookie_to_user,
      allowed_cidrs: config.security.allowed_cidrs ?? [],
    });
  }, [config?.security, form]);

  const disabled = !config;

  const validateCidrs = useCallback((_: any, value: string[]) => {
    if (!value || value.length === 0) return Promise.resolve();
    const bad = value.find((v) => !(ipv4CidrRe.test(v) || ipv6CidrRe.test(v)));
    return bad ? Promise.reject(new Error(`Invalid CIDR: "${bad}"`)) : Promise.resolve();
  }, []);

  const onSave = useCallback(async () => {
    if (!config) return message.error('No configuration loaded yet.');
    const values = await form.validateFields(); // FormShape

    const existingPin = config.security?.password_pin ?? null;

    if (
      values.password_enabled &&
      (!values.password_pin || values.password_pin.trim().length === 0)
    ) {
      return message.error('Please enter a PIN (or disable password gating).');
    }

    const toSave: AssignmentSecurityConfig = {
      password_enabled: values.password_enabled,
      password_pin: values.password_enabled ? (values.password_pin ?? '').trim() : existingPin,
      cookie_ttl_minutes: values.cookie_ttl_minutes,
      bind_cookie_to_user: values.bind_cookie_to_user,
      allowed_cidrs: values.allowed_cidrs ?? [],
    };

    try {
      await updateConfig({ security: toSave });
      message.success('Security config saved');
    } catch (e: any) {
      message.error(e?.message ?? 'Failed to save security config');
    }
  }, [config, form, updateConfig]);

  const fieldWidth = 'w-full max-w-xs';
  const textFieldWidth = 'w-full max-w-sm';

  const pinExtra = useMemo(
    () =>
      'Students must supply this PIN via the x-assignment-pin header. Staff are bypassed by guards.',
    [],
  );

  const handleGeneratePin = () => {
    form.setFieldsValue({ password_pin: generatePin(PIN_LENGTH) });
  };

  return (
    <SettingsGroup
      title="Assignment Security"
      description="Control PIN gating, cookie binding, and IP allowlists for this assignment."
    >
      <Form layout="vertical" form={form} className="space-y-6" disabled={disabled}>
        <Form.Item
          name="password_enabled"
          label="Require PIN (students only)"
          valuePropName="checked"
          className={fieldWidth}
          tooltip="When ON, students must provide the PIN to access the assignment. Staff bypass this."
        >
          <Switch />
        </Form.Item>

        <Form.Item
          noStyle
          shouldUpdate={(prev, cur) => prev.password_enabled !== cur.password_enabled}
        >
          {({ getFieldValue }) =>
            getFieldValue('password_enabled') ? (
              <Form.Item
                label="Assignment PIN"
                className={textFieldWidth}
                extra={pinExtra}
                required
              >
                <Space.Compact className="w-full">
                  <Form.Item
                    name="password_pin"
                    noStyle
                    rules={[{ required: true, message: 'Enter a PIN' }]}
                  >
                    <Input
                      className="w-full"
                      placeholder={`e.g., ${'0'.repeat(PIN_LENGTH)} ( ${PIN_LENGTH}-digit code )`}
                      maxLength={PIN_LENGTH}
                    />
                  </Form.Item>
                  <Button onClick={handleGeneratePin}>Generate</Button>
                </Space.Compact>
              </Form.Item>
            ) : null
          }
        </Form.Item>

        <Form.Item
          name="cookie_ttl_minutes"
          label="Cookie TTL (minutes)"
          className={fieldWidth}
          rules={[
            { required: true, message: 'Enter a TTL in minutes' },
            {
              type: 'number',
              min: MIN_COOKIE_TTL_MINUTES,
              message: `Must be at least ${MIN_COOKIE_TTL_MINUTES} minute`,
            },
          ]}
          extra="How long a successful unlock stays valid on the device."
        >
          <InputNumber className="w-full" min={MIN_COOKIE_TTL_MINUTES} precision={0} />
        </Form.Item>

        <Form.Item
          name="bind_cookie_to_user"
          label="Bind Cookie to User"
          valuePropName="checked"
          className={fieldWidth}
          tooltip="When ON, the unlock cookie is tied to the user id, making sharing harder."
        >
          <Switch />
        </Form.Item>

        <Form.Item
          name="allowed_cidrs"
          label="Allowed CIDRs"
          className={textFieldWidth}
          rules={[{ validator: validateCidrs }]}
          extra={cidrHelp}
        >
          <Select
            mode="tags"
            tokenSeparators={[',', ' ']}
            placeholder="Add CIDRs (leave empty to allow all)"
            className="w-full"
          />
        </Form.Item>

        <div className="pt-2">
          <AssignmentConfigActions
            primaryText="Save Security Config"
            onPrimary={onSave}
            disabled={disabled}
          />
        </div>
      </Form>
    </SettingsGroup>
  );
}
