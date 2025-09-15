import { useEffect, useMemo } from 'react';
import { Typography, Card, Alert, Space, Collapse, Table } from 'antd';
import { useHelpToc } from '@/context/HelpContext';
import { CodeEditor } from '@/components/common';
import { useBreadcrumbContext } from '@/context/BreadcrumbContext';

const { Title, Paragraph, Text } = Typography;

const toc = [
  { key: 'what', href: '#what', title: 'What does Execution control?' },
  { key: 'limits', href: '#limits', title: 'Limits & defaults' },
  { key: 'tips', href: '#tips', title: 'Tuning tips' },
  { key: 'json', href: '#json', title: 'Raw config (JSON)' },
  { key: 'trouble', href: '#trouble', title: 'Troubleshooting' }, // last
];

const DEFAULTS_JSON = `{
  "execution": {
    "timeout_secs": 10,
    "max_memory": 8589934592,
    "max_cpus": 2,
    "max_uncompressed_size": 100000000,
    "max_processes": 256
  }
}`;

const HEAVIER_SAMPLE_JSON = `{
  "execution": {
    "timeout_secs": 60,
    "max_memory": 17179869184,
    "max_cpus": 4,
    "max_uncompressed_size": 200000000,
    "max_processes": 512
  }
}`;

// ✅ Added "Options" column
const limitCols = [
  { title: 'Setting', dataIndex: 'setting', key: 'setting', width: 220 },
  { title: 'What it does', dataIndex: 'meaning', key: 'meaning' },
  { title: 'Options', dataIndex: 'options', key: 'options', width: 200 },
  { title: 'Default', dataIndex: 'def', key: 'def', width: 140 },
];

const limitRows = [
  {
    key: 't',
    setting: 'Time limit',
    meaning: 'Maximum wall-clock time for a single task run.',
    options: 'Number (seconds)',
    def: '10 s',
  },
  {
    key: 'mem',
    setting: 'Memory limit',
    meaning: 'Approximate RAM cap for the program.',
    options: 'Number (bytes) e.g. 8 GiB',
    def: '≈ 8 GiB',
  },
  {
    key: 'cpu',
    setting: 'CPU cores',
    meaning: 'How many cores the run may use.',
    options: 'Integer ≥ 1',
    def: '2',
  },
  {
    key: 'unz',
    setting: 'Max extracted size',
    meaning: 'Largest allowed size after unzipping a submission.',
    options: 'Number (bytes) e.g. 100 MB',
    def: '≈ 100 MB',
  },
  {
    key: 'proc',
    setting: 'Max processes/threads',
    meaning: 'Upper bound on processes/threads the run may spawn.',
    options: 'Integer ≥ 1',
    def: '256',
  },
];

export default function ExecutionHelp() {
  const { setBreadcrumbLabel } = useBreadcrumbContext();
  const ids = useMemo(() => toc.map((t) => t.href.slice(1)), []);

  useEffect(() => {
    setBreadcrumbLabel('help/assignments/config/execution', 'Execution');
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
          <li>Hitting any limit (time, memory, processes) fails that task run.</li>
          <li>Only raise limits if the task truly needs it — it impacts queue speed.</li>
        </ul>
      </Card>
    ),
    onMountScrollToHash: true,
  });

  return (
    <Space direction="vertical" size="small" className="w-full !p-0">
      <Title level={2} className="mb-0">
        Execution
      </Title>

      <section id="what" className="scroll-mt-24" />
      <Title level={3}>What does Execution control?</Title>
      <Paragraph className="mb-0">
        These settings cap how each task is built and run. They prevent runaway programs and keep
        grading fair and fast. The same limits apply to <b>memo generation</b> and to
        <b> student submissions</b>.
      </Paragraph>

      <section id="limits" className="scroll-mt-24" />
      <Title level={3}>Limits & defaults</Title>
      <Table
        className="mt-2"
        size="small"
        columns={limitCols}
        dataSource={limitRows}
        pagination={false}
      />

      <section id="tips" className="scroll-mt-24" />
      <Title level={3}>Tuning tips</Title>
      <ul className="list-disc pl-5">
        <li>
          Start with defaults; only raise the <b>Time limit</b> or <b>Memory limit</b> if the task
          legitimately needs it.
        </li>
        <li>
          Keep <b>Max extracted size</b> tight to avoid huge uploads.
        </li>
        <li>
          If your build spawns lots of helpers, increase <b>Max processes/threads</b> gradually
          (e.g., 256 → 512).
        </li>
        <li>
          More <b>CPU cores</b> can speed builds, but reduces overall cluster throughput — use
          sparingly.
        </li>
      </ul>

      <Alert
        className="mt-3"
        type="warning"
        showIcon
        message="Remember"
        description="If a run hits any limit (time, memory, processes), that task attempt fails and is reported in the results."
      />

      <section id="json" className="scroll-mt-24" />
      <Title level={3}>Raw config (JSON)</Title>
      <Paragraph className="mb-2">
        For reference only. The UI lets you edit these directly. Field names map to labels above:
        <Text code className="ml-1">
          timeout_secs
        </Text>{' '}
        → Time limit,&nbsp;
        <Text code>max_memory</Text> → Memory limit,&nbsp;
        <Text code>max_cpus</Text> → CPU cores,&nbsp;
        <Text code>max_uncompressed_size</Text> → Max extracted size,&nbsp;
        <Text code>max_processes</Text> → Max processes/threads.
      </Paragraph>
      <Card>
        <Paragraph className="mb-2">Defaults:</Paragraph>
        <CodeEditor
          language="json"
          value={DEFAULTS_JSON}
          height={180}
          readOnly
          minimal
          fitContent
          showLineNumbers={false}
          hideCopyButton
        />
        <Paragraph className="mt-4 mb-2">Heavier assignment example:</Paragraph>
        <CodeEditor
          language="json"
          value={HEAVIER_SAMPLE_JSON}
          height={200}
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
            label: 'Timed out',
            children: (
              <ul className="list-disc pl-5">
                <li>Check for infinite loops or blocking I/O.</li>
                <li>
                  Raise the <b>Time limit</b> if the task is legitimately slow.
                </li>
              </ul>
            ),
          },
          {
            key: 't2',
            label: 'Out of memory',
            children: (
              <ul className="list-disc pl-5">
                <li>
                  Reduce input size or memory usage; then consider raising the <b>Memory limit</b>.
                </li>
              </ul>
            ),
          },
          {
            key: 't3',
            label: 'Archive rejected (too big after unzip)',
            children: (
              <Paragraph>
                Lower submission size or raise <b>Max extracted size</b> carefully.
              </Paragraph>
            ),
          },
          {
            key: 't4',
            label: 'Too many processes/threads',
            children: (
              <Paragraph>
                Cap concurrency in your build/run or increase <b>Max processes/threads</b>.
              </Paragraph>
            ),
          },
        ]}
      />
    </Space>
  );
}
