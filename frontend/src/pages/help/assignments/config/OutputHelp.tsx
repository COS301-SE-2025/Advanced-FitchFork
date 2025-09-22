import { useEffect, useMemo } from 'react';
import { Typography, Card, Alert, Space, Collapse, Table } from 'antd';
import { useHelpToc } from '@/context/HelpContext';
import { CodeEditor } from '@/components/common';
import { useBreadcrumbContext } from '@/context/BreadcrumbContext';

const { Title, Paragraph, Text } = Typography;

const toc = [
  { key: 'what', href: '#what', title: 'What does Output control?' },
  { key: 'options', href: '#options', title: 'Options & defaults' },
  { key: 'tips', href: '#tips', title: 'Tips' },
  { key: 'json', href: '#json', title: 'Raw config (JSON)' },
  { key: 'trouble', href: '#trouble', title: 'Troubleshooting' }, // keep last
];

const DEFAULTS_JSON = `{
  "output": {
    "stdout": true,
    "stderr": false,
    "retcode": false
  }
}`;

const CAPTURE_ALL_JSON = `{
  "output": {
    "stdout": true,
    "stderr": true,
    "retcode": true
  }
}`;

// ✅ Added "Options" column
const optionCols = [
  { title: 'Setting', dataIndex: 'setting', key: 'setting', width: 260 },
  { title: 'What it does', dataIndex: 'meaning', key: 'meaning' },
  { title: 'Options', dataIndex: 'options', key: 'options', width: 140 },
  { title: 'Default', dataIndex: 'def', key: 'def', width: 120 },
];

const optionRows = [
  {
    key: 'stdout',
    setting: 'Capture standard output (stdout)',
    meaning:
      'Records text printed to stdout. This is the usual basis for comparing Student Output to Memo Output.',
    options: 'On / Off',
    def: 'On',
  },
  {
    key: 'stderr',
    setting: 'Capture error output (stderr)',
    meaning:
      'Also records text written to stderr. Turn on only if your task intentionally prints important messages there.',
    options: 'On / Off',
    def: 'Off',
  },
  {
    key: 'retcode',
    setting: 'Capture return code (exit status)',
    meaning:
      'Records the program exit status (0 = success, non-zero = error). Useful when grading on exit codes.',
    options: 'On / Off',
    def: 'Off',
  },
];

export default function OutputHelp() {
  const { setBreadcrumbLabel } = useBreadcrumbContext();
  const ids = useMemo(() => toc.map((t) => t.href.slice(1)), []);

  useEffect(() => {
    setBreadcrumbLabel('help/assignments/config/output', 'Output');
  }, []);

  useHelpToc({
    items: toc,
    ids,
    extra: (
      <Card className="mt-4" size="small" title="Quick facts" bordered>
        <ul className="list-disc pl-5">
          <li>
            Applies to both <b>Memo runs</b> and <b>Student runs</b>.
          </li>
          <li>Only what you capture here is saved and compared.</li>
          <li>
            If you change Output settings, regenerate <b>Memo Output</b> for a fair comparison.
          </li>
        </ul>
      </Card>
    ),
    onMountScrollToHash: true,
  });

  return (
    <Space direction="vertical" size="small" className="w-full !p-0">
      <Title level={2} className="mb-0">
        Output
      </Title>

      <section id="what" className="scroll-mt-24" />
      <Title level={3}>What does Output control?</Title>
      <Paragraph className="mb-0">
        Output settings decide which parts of each run are recorded: <b>stdout</b>, <b>stderr</b>,
        and the <b>return code</b>. Only the captured channels are included in results and used
        during marking.
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
          Most assignments only need <b>stdout</b>. Keep <b>stderr</b> off unless you rely on it.
        </li>
        <li>
          Enable <b>return code</b> when non-zero exits are part of the spec.
        </li>
        <li>
          Minimize noise: avoid verbose logs/ANSI color codes that make exact comparisons brittle.
        </li>
      </ul>

      <Alert
        className="mt-3"
        type="info"
        showIcon
        message="Comparison reminder"
        description="Comparators only evaluate what you capture. If stderr or return code matters, enable them here."
      />

      <section id="json" className="scroll-mt-24" />
      <Title level={3}>Raw config (JSON)</Title>
      <Paragraph className="mb-2">
        The UI manages these values, but for reference the fields are <Text code>stdout</Text>,{' '}
        <Text code>stderr</Text>, and <Text code>retcode</Text>.
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
        <Paragraph className="mt-4 mb-2">Capture everything:</Paragraph>
        <CodeEditor
          language="json"
          value={CAPTURE_ALL_JSON}
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
            label: '“Nothing was captured”',
            children: (
              <ul className="list-disc pl-5">
                <li>
                  Your program may only write to <b>stderr</b> — enable “Capture error output”.
                </li>
                <li>Or the program produced no output. Ensure your Main prints results.</li>
              </ul>
            ),
          },
          {
            key: 't2',
            label: 'Return code mismatches',
            children: (
              <ul className="list-disc pl-5">
                <li>Enable “Capture return code” if exit status is part of grading.</li>
                <li>
                  Ensure your program exits with the intended code (0 for success unless specified).
                </li>
              </ul>
            ),
          },
          {
            key: 't3',
            label: 'Unexpected diffs due to formatting',
            children: (
              <ul className="list-disc pl-5">
                <li>Trim extra logs and remove color codes; keep output deterministic.</li>
                <li>
                  End lines with <Text code>\n</Text> consistently.
                </li>
              </ul>
            ),
          },
        ]}
      />
    </Space>
  );
}
