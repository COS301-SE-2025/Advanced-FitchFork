// src/pages/help/assignments/config/Overview.tsx
import { useEffect, useMemo } from 'react';
import { Typography, Card, Space, Table, Alert, Tag, Descriptions } from 'antd';
import { useHelpToc } from '@/context/HelpContext';
import { useBreadcrumbContext } from '@/context/BreadcrumbContext';
import { useViewSlot } from '@/context/ViewSlotContext';

const { Title, Paragraph, Text } = Typography;

const toc = [
  { key: 'what', href: '#what', title: 'What is Assignment Config?' },
  { key: 'sections', href: '#sections', title: 'Sections at a glance' },
  { key: 'workflows', href: '#workflows', title: 'Common workflows' },
  { key: 'import', href: '#import', title: 'Import & Export' },
  { key: 'best', href: '#best', title: 'Best practices' },
  { key: 'trouble', href: '#trouble', title: 'Troubleshooting' }, // keep last
];

const sectionsCols = [
  { title: 'Area', dataIndex: 'area', key: 'area', width: 220 },
  { title: 'What it controls', dataIndex: 'what', key: 'what' },
  { title: 'Go to', dataIndex: 'link', key: 'link', width: 220 },
];

const sectionsRows = [
  {
    key: 'project',
    area: 'Language & Mode',
    what: 'Programming language and how students submit (Manual, RNG, Code Coverage, GATLAM).',
    link: <a href="/help/assignments/config/project">Open Language & Mode →</a>,
  },
  {
    key: 'execution',
    area: 'Execution',
    what: 'Time limit, memory, CPU cores, unzip size, and process/thread caps for each run.',
    link: <a href="/help/assignments/config/execution">Open Execution →</a>,
  },
  {
    key: 'marking',
    area: 'Marking',
    what: 'Marking scheme, feedback style, pass mark, attempt limits, delimiter, disallowed code.',
    link: <a href="/help/assignments/config/marking">Open Marking →</a>,
  },
  {
    key: 'security',
    area: 'Security',
    what: 'Optional unlock PIN, cookie duration/binding, and IP allowlists.',
    link: <a href="/help/assignments/config/security">Open Security →</a>,
  },
  {
    key: 'gatlam',
    area: 'GATLAM',
    what: 'Genetic search parameters and interpreter checks (return codes, runtime bounds, forbidden outputs).',
    link: <a href="/help/assignments/config/gatlam">Open GATLAM →</a>,
  },
  {
    key: 'coverage',
    area: 'Code Coverage',
    what: 'Coverage threshold required to pass when using the Code Coverage mode.',
    link: <a href="/help/assignments/code-coverage">Open Code Coverage →</a>,
  },
];

export default function ConfigOverviewHelp() {
  const { setBreadcrumbLabel } = useBreadcrumbContext();
  const ids = useMemo(() => toc.map((t) => t.href.slice(1)), []);
  const { setValue, setBackTo } = useViewSlot();

  useEffect(() => {
    setValue(
      <Typography.Text className="text-base font-medium text-gray-900 dark:text-gray-100 truncate">
        Assignment Config Overview
      </Typography.Text>,
    );
    setBackTo('/help');
  }, []);

  useEffect(() => {
    setBreadcrumbLabel('help/assignments/config', 'Assignment Config');
  }, []);

  useHelpToc({
    items: toc,
    ids,
    extra: (
      <Card className="mt-4" size="small" title="Quick facts" bordered>
        <ul className="list-disc pl-5">
          <li>
            Edit everything in <Text code>Assignments → Config</Text>.
          </li>
          <li>
            Changes apply to new runs. If you change the <b>delimiter</b>, regenerate <b>Memo
            Output</b>.
          </li>
          <li>Safe defaults are provided; raise limits only when needed.</li>
        </ul>
      </Card>
    ),
    onMountScrollToHash: true,
  });

  return (
    <Space direction="vertical" size="small" className="w-full !p-0">
      <Title level={2} className="mb-0">
        Assignment Config
      </Title>

      <section id="what" className="scroll-mt-24" />
      <Title level={3}>What is Assignment Config?</Title>
      <Paragraph className="mb-0">
        The control panel for how your assignment builds, runs, captures output, and is graded. It’s
        split into:
        <Tag className="ml-2">Language & Mode</Tag>
        <Tag>Execution</Tag>
        <Tag>Marking</Tag>
        <Tag>Security</Tag>
        <Tag>GATLAM</Tag>
        <Tag>Code Coverage</Tag>
      </Paragraph>

      <section id="sections" className="scroll-mt-24" />
      <Title level={3}>Sections at a glance</Title>

      {/* md+ : normal table */}
      <div className="hidden md:block">
        <Table
          size="small"
          columns={sectionsCols}
          dataSource={sectionsRows}
          pagination={false}
          scroll={{ x: true }}
        />
      </div>

      {/* <md : cards (no extra shadows) */}
      <div className="block md:hidden !space-y-3">
        {sectionsRows.map((r) => (
          <Card
            key={r.key}
            size="small"
            title={<div className="text-base font-semibold truncate">{r.area}</div>}
          >
            <div className="text-xs uppercase tracking-wide text-gray-500 dark:text-gray-400 mb-1">
              What it controls
            </div>
            <div className="text-sm text-gray-900 dark:text-gray-100 mb-2">{r.what}</div>

            {r.link && (
              <div className="text-xs uppercase tracking-wide text-gray-500 dark:text-gray-400 mb-1">
                Go to
              </div>
            )}
            <div className="text-sm text-gray-900 dark:text-gray-100">{r.link}</div>
          </Card>
        ))}
      </div>

      <section id="workflows" className="scroll-mt-24" />
      <Title level={3}>Common workflows</Title>
      <Descriptions bordered size="middle" column={1} className="mt-2">
        <Descriptions.Item label="New assignment (fast)">
          Pick your <b>Language & Mode</b> → keep <b>Execution</b> defaults → keep <b>Marking</b>
          as Exact (pass mark 50%).
        </Descriptions.Item>
        <Descriptions.Item label="Time- or memory-heavy tasks">
          Increase <b>Time limit</b> and/or <b>Memory</b> gradually to keep the queue healthy.
        </Descriptions.Item>
        <Descriptions.Item label="Restrict access">
          Use <b>Security</b> (PIN, cookie binding, IP allowlist) for labs or invigilated sessions.
        </Descriptions.Item>
        <Descriptions.Item label="Search for edge cases">
          Configure <b>GATLAM</b> and use labeled subsections (<Text code>###</Text>) in Main for
          precise comparisons.
        </Descriptions.Item>
      </Descriptions>

      <section id="import" className="scroll-mt-24" />
      <Title level={3}>Import & Export</Title>
      <Paragraph className="mb-0">
        In <Text code>Assignments → Config</Text> you can <b>Export</b> the current settings and{' '}
        <b>Import</b> them into another assignment. Importing replaces the existing config. After
        import, regenerate <b>Memo Output</b> if you changed delimiters or mark allocation.
      </Paragraph>

      <section id="best" className="scroll-mt-24" />
      <Title level={3}>Best practices</Title>
      <ul className="list-disc pl-5">
        <li>Keep outputs deterministic and minimal to simplify marking.</li>
        <li>
          Use <b>Disallowed code</b> in Marking to block banned libs/patterns.
        </li>
        <li>Prefer small, measured increases to time/memory/parallelism.</li>
      </ul>

      {/* Troubleshooting LAST */}
      <section id="trouble" className="scroll-mt-24" />
      <Title level={3}>Troubleshooting</Title>
      <Space direction="vertical" size="small">
        <Alert
          showIcon
          type="warning"
          message="“My changes didn’t affect marks”"
          description="If you changed delimiters or mark allocation, regenerate Memo Output so comparisons use the new settings."
        />
        <Alert
          showIcon
          type="info"
          message="“Students can’t access the assignment”"
          description="Check Security: PIN enabled? Cookie expired? IP allowlist too strict?"
        />
      </Space>
    </Space>
  );
}
