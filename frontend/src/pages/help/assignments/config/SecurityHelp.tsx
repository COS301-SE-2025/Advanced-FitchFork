// src/pages/help/assignments/config/SecurityHelp.tsx
import { useEffect, useMemo } from 'react';
import { Typography, Card, Space, Collapse, Table, Alert } from 'antd';
import { useHelpToc } from '@/context/HelpContext';
import { CodeEditor } from '@/components/common';
import { useBreadcrumbContext } from '@/context/BreadcrumbContext';

const { Title, Paragraph, Text } = Typography;

const toc = [
  { key: 'what', href: '#what', title: 'What does Security control?' },
  { key: 'options', href: '#options', title: 'Options & defaults' },
  { key: 'tips', href: '#tips', title: 'Tips' },
  { key: 'json', href: '#json', title: 'Raw config (JSON)' },
  { key: 'trouble', href: '#trouble', title: 'Troubleshooting' },
];

const DEFAULTS_JSON = `{
  "security": {
    "password_enabled": false,
    "password_pin": null,
    "cookie_ttl_minutes": 480,
    "bind_cookie_to_user": true,
    "allowed_cidrs": []
  }
}`;

const EXAMPLE_STRICT_JSON = `{
  "security": {
    "password_enabled": true,
    "password_pin": "4321",
    "cookie_ttl_minutes": 120,
    "bind_cookie_to_user": true,
    "allowed_cidrs": ["10.0.0.0/24", "196.21.0.0/16"]
  }
}`;

// Human-friendly table
const optionCols = [
  { title: 'Setting', dataIndex: 'setting', key: 'setting', width: 260 },
  { title: 'What it does', dataIndex: 'meaning', key: 'meaning' },
  { title: 'Options / Format', dataIndex: 'options', key: 'options', width: 260 },
  { title: 'Default', dataIndex: 'def', key: 'def', width: 140 },
];

const optionRows = [
  {
    key: 'pin',
    setting: 'Unlock with PIN',
    meaning:
      'Requires students to enter a PIN once per device/session before they can view or submit the assignment.',
    options: 'On / Off; PIN is a short string (e.g., "4321").',
    def: 'Off (no PIN)',
  },
  {
    key: 'ttl',
    setting: 'Cookie lifetime',
    meaning:
      'How long the unlock stays valid on a device before the student is asked to unlock again.',
    options: 'Minutes (e.g., 480 = 8 hours).',
    def: '480 minutes',
  },
  {
    key: 'bind',
    setting: 'Bind cookie to user',
    meaning:
      'Ties the unlock cookie to the specific user so it cannot be shared by copying the cookie.',
    options: 'On / Off',
    def: 'On',
  },
  {
    key: 'cidr',
    setting: 'Allowed IP ranges (CIDR)',
    meaning:
      'Restrict access to specific networks (e.g., labs/campus). Leave empty for no IP restriction.',
    options: 'List of CIDRs (e.g., "10.0.0.0/24").',
    def: 'None (no restriction)',
  },
];

export default function SecurityHelp() {
  const { setBreadcrumbLabel } = useBreadcrumbContext();
  const ids = useMemo(() => toc.map((t) => t.href.slice(1)), []);

  useEffect(() => {
    setBreadcrumbLabel('help/assignments/config/security', 'Security');
  }, []);

  useHelpToc({
    items: toc,
    ids,
    extra: (
      <Card className="mt-4" size="small" title="Quick facts" bordered>
        <ul className="list-disc pl-5">
          <li>Use a PIN for supervised tests or on-site labs.</li>
          <li>Cookie lifetime controls how often students need to unlock again.</li>
          <li>IP allowlists help keep access to known networks only.</li>
        </ul>
      </Card>
    ),
    onMountScrollToHash: true,
  });

  return (
    <Space direction="vertical" size="small" className="w-full !p-0">
      <Title level={2} className="mb-0">
        Security
      </Title>

      <section id="what" className="scroll-mt-24" />
      <Title level={3}>What does Security control?</Title>
      <Paragraph className="mb-0">
        Security settings gate access to an assignment. You can require a one-time <b>PIN unlock</b>
        , limit access by <b>IP ranges</b>, and control how long an unlock remains valid on a
        device.
      </Paragraph>

      <section id="options" className="scroll-mt-24" />
      <Title level={3}>Options & defaults</Title>
      <Table
        className="mt-2"
        size="small"
        columns={optionCols}
        dataSource={optionRows}
        pagination={false}
      />

      <section id="tips" className="scroll-mt-24" />
      <Title level={3}>Tips</Title>
      <ul className="list-disc pl-5">
        <li>
          For in-lab assessments, enable <b>Unlock with PIN</b> and set a sensible{' '}
          <b>Cookie lifetime</b> (e.g., the session length).
        </li>
        <li>
          Keep <b>Bind cookie to user</b> on to prevent one unlock from being shared.
        </li>
        <li>
          Use <b>Allowed IP ranges</b> with care — include all required networks (e.g., VPN ranges)
          to avoid locking out legitimate students.
        </li>
      </ul>

      <Alert
        className="mt-2"
        type="info"
        showIcon
        message="Note"
        description="If a student is on the wrong network or the cookie expired, they'll be prompted to unlock again."
      />

      <section id="json" className="scroll-mt-24" />
      <Title level={3}>Raw config (JSON)</Title>
      <Paragraph className="mb-2">
        The UI manages these, but here’s how fields map: <Text code>password_enabled</Text> (Unlock
        with PIN), <Text code>password_pin</Text> (PIN value), <Text code>cookie_ttl_minutes</Text>{' '}
        (Cookie lifetime), <Text code>bind_cookie_to_user</Text> (Bind cookie to user),{' '}
        <Text code>allowed_cidrs</Text> (IP allowlist).
      </Paragraph>
      <Card>
        <Paragraph className="mb-2">Defaults:</Paragraph>
        <CodeEditor
          language="json"
          value={DEFAULTS_JSON}
          height={160}
          readOnly
          minimal
          fitContent
          showLineNumbers={false}
          hideCopyButton
        />
        <Paragraph className="mt-4 mb-2">Example (PIN + campus networks):</Paragraph>
        <CodeEditor
          language="json"
          value={EXAMPLE_STRICT_JSON}
          height={180}
          readOnly
          minimal
          fitContent
          showLineNumbers={false}
          hideCopyButton
        />
      </Card>

      {/* Troubleshooting LAST */}
      <section id="trouble" className="scroll-mt-24" />
      <Title level={3}>Troubleshooting</Title>
      <Collapse
        items={[
          {
            key: 't1',
            label: 'Students stuck at unlock screen',
            children: (
              <ul className="list-disc pl-5">
                <li>
                  Confirm <b>Unlock with PIN</b> is enabled and the current PIN is communicated.
                </li>
                <li>
                  Check <b>Allowed IP ranges</b> cover the student’s network (including VPN).
                </li>
              </ul>
            ),
          },
          {
            key: 't2',
            label: 'Unlock keeps reappearing',
            children: (
              <ul className="list-disc pl-5">
                <li>
                  The <b>Cookie lifetime</b> may be too short; increase it.
                </li>
                <li>The browser may be blocking cookies — ask the student to allow them.</li>
              </ul>
            ),
          },
          {
            key: 't3',
            label: 'Access denied off-campus',
            children: (
              <Paragraph className="mb-0">
                Add the off-campus/VPN ranges to <b>Allowed IP ranges</b>, or clear the list to
                disable the restriction.
              </Paragraph>
            ),
          },
        ]}
      />
    </Space>
  );
}
