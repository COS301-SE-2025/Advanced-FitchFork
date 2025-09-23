// src/pages/help/assignments/config/ProjectHelp.tsx
import { useEffect, useMemo } from 'react';
import { Typography, Card, Space, Collapse, Table } from 'antd';
import { useHelpToc } from '@/context/HelpContext';
import { CodeEditor } from '@/components/common';
import { useBreadcrumbContext } from '@/context/BreadcrumbContext';
import { useViewSlot } from '@/context/ViewSlotContext';

const { Title, Paragraph, Text } = Typography;

const toc = [
  { key: 'what', href: '#what', title: 'What does Project control?' },
  { key: 'options', href: '#options', title: 'Options & defaults' },
  { key: 'tips', href: '#tips', title: 'Tips' },
  { key: 'json', href: '#json', title: 'Raw config (JSON)' },
  { key: 'trouble', href: '#trouble', title: 'Troubleshooting' },
];

const DEFAULTS_JSON = `{
  "project": {
    "language": "cpp",
    "submission_mode": "manual"
  }
}`;

const CODECOV_SAMPLE_JSON = `{
  "project": {
    "language": "java",
    "submission_mode": "codecoverage"
  }
}`;

// Human-friendly options table
const optionCols = [
  { title: 'Setting', dataIndex: 'setting', key: 'setting', width: 240 },
  { title: 'What it does', dataIndex: 'meaning', key: 'meaning' },
  { title: 'Options', dataIndex: 'options', key: 'options', width: 280 },
  { title: 'Default', dataIndex: 'def', key: 'def', width: 140 },
];

const optionRows = [
  {
    key: 'lang',
    setting: 'Language',
    meaning:
      'Sets the language for building and running your tasks. This also decides the expected entry filename in your Main archive.',
    options: 'Java • C++',
    def: 'C++',
  },
  {
    key: 'mode',
    setting: 'Submission mode',
    meaning:
      'Controls how students submit or how runs are produced. Manual is the normal mode. Others are advanced.',
    options: 'Manual • GATLAM • RNG • Code Coverage',
    def: 'Manual',
  },
];

export default function ProjectHelp() {
  const { setBreadcrumbLabel } = useBreadcrumbContext();
  const ids = useMemo(() => toc.map((t) => t.href.slice(1)), []);
  const { setValue, setBackTo } = useViewSlot();

  useEffect(() => {
    setValue(
      <Typography.Text className="text-base font-medium text-gray-900 dark:text-gray-100 truncate">
        Project Config
      </Typography.Text>,
    );
    setBackTo('/help');
  }, []);

  useEffect(() => {
    // Breadcrumb uses the Help route key
    setBreadcrumbLabel('help/assignments/config/project', 'Language & Mode');
  }, []);

  useHelpToc({
    items: toc,
    ids,
    extra: (
      <Card className="mt-4" size="small" title="Quick facts" bordered>
        <ul className="list-disc pl-5">
          <li>
            <b>Language</b> sets compiler/runtime expectations (and the entry filename in{' '}
            <Text code>Main</Text>).
          </li>
          <li>
            <b>Submission mode</b> switches how runs are generated and/or evaluated (advanced modes
            optional).
          </li>
        </ul>
      </Card>
    ),
    onMountScrollToHash: true,
  });

  return (
    <Space direction="vertical" size="small" className="w-full !p-0">
      <Title level={2} className="mb-0">
        Language &amp; Mode
      </Title>

      <section id="what" className="scroll-mt-24" />
      <Title level={3}>What does Project control?</Title>
      <Paragraph className="mb-0">
        The Project settings define the <b>language</b> your assignment is built in and the{' '}
        <b>submission mode</b> used for grading. Pick the language that matches your Main/Makefile,
        then choose how students submit (Manual) or enable an advanced mode when needed.
      </Paragraph>

      <section id="options" className="scroll-mt-24" />
      <Title level={3}>Options & defaults</Title>

      {/* md+ : normal table */}
      <div className="hidden md:block">
        <Table
          className="mt-2"
          size="small"
          columns={optionCols}
          dataSource={optionRows}
          pagination={false}
          scroll={{ x: true }}
        />
      </div>

      {/* <md : cards (no extra shadows) */}
      <div className="block md:hidden mt-2 !space-y-3">
        {optionRows.map((r) => (
          <Card
            key={r.key}
            size="small"
            title={<div className="text-base font-semibold truncate">{r.setting}</div>}
          >
            <div className="text-xs uppercase tracking-wide text-gray-500 dark:text-gray-400 mb-1">
              What it does
            </div>
            <div className="text-sm text-gray-900 dark:text-gray-100 mb-2">{r.meaning}</div>

            <div className="text-xs uppercase tracking-wide text-gray-500 dark:text-gray-400 mb-1">
              Options
            </div>
            <div className="text-sm text-gray-900 dark:text-gray-100 mb-2">{r.options}</div>

            <div className="text-xs uppercase tracking-wide text-gray-500 dark:text-gray-400 mb-1">
              Default
            </div>
            <div className="text-sm text-gray-900 dark:text-gray-100">{r.def}</div>
          </Card>
        ))}
      </div>

      <section id="tips" className="scroll-mt-24" />
      <Title level={3}>Tips</Title>
      <ul className="list-disc pl-5">
        <li>
          Changing <b>Language</b> usually means re-checking your{' '}
          <a href="/help/assignments/files/main-files">Main archive</a> and{' '}
          <a href="/help/assignments/files/makefile">Makefile</a> (entry filename, toolchain).
        </li>
        <li>
          <b>Manual</b> is the standard mode for most courses. Use <b>Code Coverage</b> if you also
          require a coverage threshold (configured under{' '}
          <a href="/help/assignments/code-coverage">Code Coverage</a>).
        </li>
        <li>
          <b>GATLAM</b> and <b>RNG</b> are advanced/experimental. Only enable them if your
          assignment has been set up to use those workflows.
        </li>
      </ul>

      <section id="json" className="scroll-mt-24" />
      <Title level={3}>Raw config (JSON)</Title>
      <Paragraph className="mb-2">
        The UI manages these, but here’s the mapping: <Text code>project.language</Text> and{' '}
        <Text code>project.submission_mode</Text>. Values are lowercase.
      </Paragraph>
      <Card>
        <Paragraph className="mb-2">Defaults:</Paragraph>
        <CodeEditor
          language="json"
          value={DEFAULTS_JSON}
          height={120}
          readOnly
          minimal
          fitContent
          showLineNumbers={false}
          hideCopyButton
        />
        <Paragraph className="mt-4 mb-2">Example (Java + Code Coverage):</Paragraph>
        <CodeEditor
          language="json"
          value={CODECOV_SAMPLE_JSON}
          height={120}
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
            label: '“Main not found / wrong filename”',
            children: (
              <Paragraph className="mb-0">
                Confirm <b>Language</b> matches your Main entry file (e.g.,{' '}
                <Text code>Main.cpp</Text> vs <Text code>Main.java</Text>) and re-upload the correct
                archive.
              </Paragraph>
            ),
          },
          {
            key: 't2',
            label: 'Build tools mismatch after switching language',
            children: (
              <Paragraph className="mb-0">
                Update your <a href="/help/assignments/files/makefile">Makefile</a> and ensure the
                toolchain matches the selected language.
              </Paragraph>
            ),
          },
          {
            key: 't3',
            label: 'Mode-specific features not working',
            children: (
              <ul className="list-disc pl-5">
                <li>
                  <b>Code Coverage</b>: set the threshold under{' '}
                  <a href="/help/assignments/code-coverage">Code Coverage</a>.
                </li>
                <li>
                  <b>GATLAM / RNG</b>: ensure the rest of the config is prepared for these advanced
                  flows.
                </li>
              </ul>
            ),
          },
        ]}
      />
    </Space>
  );
}
