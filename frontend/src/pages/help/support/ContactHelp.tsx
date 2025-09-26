import { useMemo } from 'react';
import { Alert, List, Space, Typography } from 'antd';
import { MailOutlined } from '@ant-design/icons';
import { useHelpToc } from '@/context/HelpContext';
import { useHelpPageHeader } from '@/pages/help/useHelpPageHeader';

const { Title, Paragraph, Text, Link } = Typography;

const toc = [
  { key: 'when', href: '#when', title: 'When to contact support' },
  { key: 'info', href: '#info', title: 'Information to include' },
  { key: 'escalation', href: '#escalation', title: 'Urgent escalation channels' },
  { key: 'self-service', href: '#self-service', title: 'Self-service checklist' },
];

const selfServiceItems = [
  'Check system monitoring for current outages or degraded services.',
  'Confirm you can reproduce the issue in a private browser window.',
  'Grab the exact module and assignment ID from the URL when reporting marking issues.',
  'Download the relevant submission or memo artefacts if the problem relates to grading.',
];

export default function HelpContactSupport() {
  useHelpPageHeader('support/contact', { label: 'Contact Support' });
  const ids = useMemo(() => toc.map((t) => t.href.slice(1)), []);

  useHelpToc({ items: toc, ids, onMountScrollToHash: true });

  return (
    <Space direction="vertical" size="large" className="w-full !p-0">
      <div>
        <Title level={2} className="mb-2">
          Contact Support
        </Title>
        <Paragraph className="mb-0 text-gray-600 dark:text-gray-300">
          Need a hand beyond the docs? Reach the FitchFork support desk with the details below so we
          can reproduce the issue quickly and provide a fix or workaround.
        </Paragraph>
      </div>

      <section id="when" className="scroll-mt-32">
        <Title level={3}>When to contact support</Title>
        <List
          bordered
          size="small"
          dataSource={[
            'Platform availability problems such as timeouts, 5xx responses, or missing dashboards.',
            'Submission or memo processing failures that persist after a retry.',
            'Role or permission issues not resolved via the module Personnel tool.',
            'Security concerns, suspected account compromise, or incorrect access.',
          ]}
          renderItem={(item) => <List.Item className="text-gray-700 dark:text-gray-200">{item}</List.Item>}
        />
      </section>

      <section id="info" className="scroll-mt-32">
        <Title level={3}>Information to include</Title>
        <Paragraph>
          Email <Link href="mailto:support@fitchfork.app">support@fitchfork.app</Link> or use the
          in-product feedback panel. Include the following so we can investigate straight away:
        </Paragraph>
        <List
          size="small"
          dataSource={[
            'Your name, role, and institution.',
            'Module and assignment IDs (copy the numbers from the URL).',
            'Exact timestamp and timezone of the incident.',
            'Screenshots or error messages. For submission failures, attach the submission ID.',
          ]}
          renderItem={(item) => <List.Item className="text-gray-700 dark:text-gray-200">{item}</List.Item>}
        />
        <Alert
          className="mt-4"
          type="info"
          showIcon
          message="Template"
          description={
            <Paragraph className="mb-0">
              Paste this into your message:
              <pre className="mt-2 rounded-md bg-gray-50 dark:bg-gray-900 p-3 text-xs overflow-auto">
{`Subject: [FitchFork] Support request
Module: <code / name>
Assignment / Session: <id>
Issue: <short summary>
Steps to reproduce:
1.
2.
3.
Expected vs actual result:
Attachments: <screenshots / IDs>`}
              </pre>
            </Paragraph>
          }
        />
      </section>

      <section id="escalation" className="scroll-mt-32">
        <Title level={3}>Urgent escalation channels</Title>
        <Paragraph>
          For outage-level incidents contact the on-call number listed in your support agreement or
          page the team via the Slack <Text code>#fitchfork-support</Text> channel. Include the same
          context as the email template so logs can be traced quickly.
        </Paragraph>
      </section>

      <section id="self-service" className="scroll-mt-32">
        <Title level={3}>Self-service checklist</Title>
        <Paragraph>
          Before raising a ticket, run through these quick checks. They solve the majority of
          day-to-day issues and reduce back-and-forth once the ticket is open.
        </Paragraph>
        <List
          dataSource={selfServiceItems}
          renderItem={(item) => <List.Item className="text-gray-700 dark:text-gray-200">{item}</List.Item>}
        />
        <Alert
          className="mt-4"
          type="success"
          showIcon
          icon={<MailOutlined />}
          message="Ready to send"
          description="Once you have your summary, send it to support@fitchfork.app or submit it via the Help → Contact form."
        />
      </section>
    </Space>
  );
}
