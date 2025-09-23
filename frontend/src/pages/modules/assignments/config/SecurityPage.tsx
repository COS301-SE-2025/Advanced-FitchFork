import { useEffect, useCallback, useMemo } from 'react';
import { Form, Input, Select, Typography, Switch, InputNumber, Button, Space } from 'antd';
import { AimOutlined } from '@ant-design/icons';
import SettingsGroup from '@/components/SettingsGroup';
import { useViewSlot } from '@/context/ViewSlotContext';
import { useAssignment } from '@/context/AssignmentContext';
import { message } from '@/utils/message';
import AssignmentConfigActions from '@/components/assignments/AssignmentConfigActions';
import type { AssignmentSecurityConfig } from '@/types/modules/assignments/config';
import Tip from '@/components/common/Tip';

// 🆕 network helpers (you said you added these)
import { getCurrentIpAsCidr, asSingleHostCIDR, isIPv4, isIPv6 } from '@/utils/network';

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
  Array.from(
    { length: len },
    () => PIN_CHARS[Math.floor(Math.random() * Math.random() * PIN_CHARS.length)],
  ).join('');

export default function SecurityPage() {
  const { setValue } = useViewSlot();
  const { config, updateConfig } = useAssignment();
  const [form] = Form.useForm<FormShape>();

  useEffect(() => {
    setValue(
      <Space align="center" size={6} className="flex-wrap">
        <Typography.Text className="text-base font-medium text-gray-900 dark:text-gray-100 truncate">
          Security
        </Typography.Text>
        <Tip iconOnly newTab to="/help/assignments/config/security#what" text="Security help" />
      </Space>,
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

  // ── CIDR normalization helpers
  const looksLikeCidr = (s: string) => /\/\d{1,3}$/.test(s.trim());

  const normalizeOne = (raw: string): string | null => {
    const v = raw.trim();
    if (!v) return null;
    // bare IP → /32 or /128
    if (!looksLikeCidr(v)) {
      if (isIPv4(v) || isIPv6(v)) {
        try {
          return asSingleHostCIDR(v);
        } catch {
          return null;
        }
      }
      return null;
    }
    // rough mask sanity
    const [ip, maskStr] = v.split('/');
    const mask = Number(maskStr);
    if (isIPv4(ip)) return mask >= 0 && mask <= 32 ? `${ip}/${mask}` : null;
    if (isIPv6(ip)) return mask >= 0 && mask <= 128 ? `${ip}/${mask}` : null;
    return null;
  };

  const normalizeMany = (vals: (string | number)[] | undefined): string[] => {
    const out: string[] = [];
    (vals ?? []).forEach((x) => {
      const s = String(x).trim();
      if (!s) return;
      const norm = normalizeOne(s);
      if (norm && !out.includes(norm)) out.push(norm);
    });
    return out;
  };

  const validateCidrs = useCallback((_: any, value: string[]) => {
    if (!value || value.length === 0) return Promise.resolve();
    const bad = value.find((v) => !(ipv4CidrRe.test(v) || ipv6CidrRe.test(v)));
    return bad ? Promise.reject(new Error(`Invalid CIDR: "${bad}"`)) : Promise.resolve();
  }, []);

  // ── Save
  const onSave = useCallback(async () => {
    if (!config) return message.error('No configuration loaded yet.');
    // Make sure the select's values are normalized before save
    const rawValues = await form.validateFields(); // FormShape
    const normalizedCidrs = normalizeMany(rawValues.allowed_cidrs);
    form.setFieldsValue({ allowed_cidrs: normalizedCidrs });

    const values = await form.validateFields(); // re-validate after normalization

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

  // ── Add My IP handler
  const onAddMyIp = async () => {
    try {
      const cidr = await getCurrentIpAsCidr(); // returns /32 or /128
      const current = form.getFieldValue('allowed_cidrs') as string[] | undefined;
      const next = normalizeMany([...(current ?? []), cidr]);
      form.setFieldsValue({ allowed_cidrs: next });
      message.success(`Added ${cidr}`);
    } catch {
      message.error('Could not detect your public IP.');
    }
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

        {/* ── Allowed CIDRs with “Add my IP” ───────────────────── */}
        <Form.Item label="Allowed CIDRs" className={textFieldWidth} extra={cidrHelp}>
          <Space direction="vertical" className="w-full">
            <Space.Compact className="w-full">
              {/* Keep validation on the inner Form.Item that actually owns the value */}
              <Form.Item name="allowed_cidrs" noStyle rules={[{ validator: validateCidrs }]}>
                <Select
                  mode="tags"
                  tokenSeparators={[',', ' ']}
                  placeholder="Add CIDRs (leave empty to allow all)"
                  className="w-full"
                  // Normalize + dedupe on change
                  onChange={(vals) => {
                    const next = normalizeMany(vals as (string | number)[]);
                    form.setFieldsValue({ allowed_cidrs: next });
                  }}
                />
              </Form.Item>

              <Button icon={<AimOutlined />} onClick={onAddMyIp}>
                Add my IP
              </Button>
            </Space.Compact>

            <Typography.Text type="secondary" className="text-xs">
              Bare IPs are auto-converted to single-host CIDR (<code>/32</code> for IPv4,{' '}
              <code>/128</code> for IPv6).
            </Typography.Text>
          </Space>
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
