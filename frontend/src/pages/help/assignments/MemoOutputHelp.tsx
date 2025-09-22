// src/pages/help/assignments/MemoOutputHelp.tsx
import { useEffect, useMemo } from 'react';
import { Typography, Card, Descriptions, Tag, Alert, Space, Collapse, Table } from 'antd';
import { useHelpToc } from '@/context/HelpContext';
import { useBreadcrumbContext } from '@/context/BreadcrumbContext';
import { CodeEditor } from '@/components/common';

const { Title, Paragraph, Text } = Typography;

const toc = [
  { key: 'what', href: '#what', title: 'What is Memo Output?' },
  { key: 'generate', href: '#generate', title: 'Generate Memo Output' },
  { key: 'view', href: '#view', title: 'View under Tasks' },
  { key: 'how', href: '#how', title: 'How it’s produced' },
  { key: 'labels', href: '#labels', title: 'Subsections & labels' },
  { key: 'regen', href: '#regen', title: 'When to regenerate' },
  { key: 'tips', href: '#tips', title: 'Tips' },
  { key: 'trouble', href: '#trouble', title: 'Troubleshooting' }, // keep last
];

const SAMPLE_STDOUT = `&-=-& Step 1
42
&-=-& Step 2
OK
Retcode: 0
`;

const SAMPLE_LABELED_TASKS = `&-=-& task1/Step 1
hello
&-=-& task1/Step 2
12

&-=-& task2/Step 1
init
&-=-& task2/Step 2
done
`;

const regenCols = [
  { title: 'Change you make', dataIndex: 'change', key: 'change', width: 280 },
  { title: 'Regenerate Memo Output?', dataIndex: 'need', key: 'need', width: 200 },
  { title: 'Why', dataIndex: 'why', key: 'why' },
];

const regenRows = [
  {
    key: 'main',
    change: 'Main file/archive',
    need: <Tag color="red">Yes</Tag>,
    why: 'It changes what runs and prints.',
  },
  {
    key: 'memo',
    change: 'Memo (reference) files',
    need: <Tag color="red">Yes</Tag>,
    why: 'New reference code ⇒ new expected output.',
  },
  {
    key: 'make',
    change: 'Makefile / build or run commands',
    need: <Tag color="red">Yes</Tag>,
    why: 'Different build/run steps change output.',
  },
  {
    key: 'output',
    change: 'Output settings (stdout / stderr / return code)',
    need: <Tag color="red">Yes</Tag>,
    why: 'You changed what is recorded for comparison.',
  },
  {
    key: 'delim',
    change: 'Delimiter (subsection label marker)',
    need: <Tag color="red">Yes</Tag>,
    why: 'Labels change; comparisons are label-aware.',
  },
  {
    key: 'tasks',
    change: 'Tasks/arguments used by Main',
    need: <Tag color="red">Yes</Tag>,
    why: 'Different tasks produce different sections.',
  },
  {
    key: 'marking',
    change: 'Marking scheme (Exact/Percentage/Regex)',
    need: <Tag>Usually No</Tag>,
    why: 'It changes how we compare, not the reference output.',
  },
  {
    key: 'limits',
    change: 'Execution limits (time, memory, CPUs)',
    need: <Tag>No*</Tag>,
    why: 'Reference text is the same if it still completes; regenerate if earlier runs timed out.',
  },
  {
    key: 'spec',
    change: 'Specification ZIP (student skeleton)',
    need: <Tag>No</Tag>,
    why: 'Spec is for starter code & similarity checks, not memo generation.',
  },
];

export default function MemoOutputHelp() {
  const { setBreadcrumbLabel } = useBreadcrumbContext();
  const ids = useMemo(() => toc.map((t) => t.href.slice(1)), []);

  useEffect(() => {
    setBreadcrumbLabel('help/assignments/memo-output', 'Memo Output');
  }, []);

  useHelpToc({
    items: toc,
    ids,
    extra: (
      <Card className="mt-4" size="small" title="Quick facts" bordered>
        <ul className="list-disc pl-5">
          <li>
            Memo Output is the <b>reference result</b> from <b>Main + Memo</b>.
          </li>
          <li>Generate it from the assignment page (top-right button).</li>
          <li>
            View it under <b>Tasks → select a task → choose a subsection</b>.
          </li>
        </ul>
      </Card>
    ),
    onMountScrollToHash: true,
  });

  return (
    <Space direction="vertical" size="small" className="w-full !p-0">
      <Title level={2} className="mb-0">
        Memo Output
      </Title>

      <section id="what" className="scroll-mt-24" />
      <Title level={3}>What is Memo Output?</Title>
      <Paragraph className="mb-0">
        The <b>reference output</b> generated from your official solution (Memo). During grading,
        each student run is compared to this result using your chosen marking scheme.
      </Paragraph>

      <section id="generate" className="scroll-mt-24" />
      <Title level={3}>Generate Memo Output</Title>
      <ol className="list-decimal pl-5">
        <li>
          Open the <b>assignment</b>.
        </li>
        <li>
          Click the <b>Generate Memo Output</b> button in the <b>top-right</b>.
        </li>
        <li>Wait for the run to finish; the reference is stored for comparisons.</li>
      </ol>
      <Alert
        className="mt-2"
        type="info"
        showIcon
        message="Heads-up"
        description="If you change Main, Memo files, Makefile, Output settings, the delimiter, or task arguments, regenerate Memo Output."
      />

      <section id="view" className="scroll-mt-24" />
      <Title level={3}>View under Tasks</Title>
      <Paragraph className="mb-1">To view memo results for a specific slice:</Paragraph>
      <ol className="list-decimal pl-5">
        <li>
          Open the assignment’s <b>Tasks</b> section.
        </li>
        <li>
          Select the <b>specific task</b> (e.g., <Text code>task1</Text>, <Text code>task2</Text>).
        </li>
        <li>
          Choose a <b>Subsection</b> label (the text printed after your delimiter in Main).
        </li>
      </ol>
      <Paragraph className="mt-1">
        You’ll see the <b>Memo Output</b> for that task/subsection.{' '}
        <b>Each task’s memo output is stored in its own text file</b> at generation time.
      </Paragraph>

      <section id="how" className="scroll-mt-24" />
      <Title level={3}>How it’s produced</Title>
      <Descriptions bordered size="middle" column={1} className="mt-2">
        <Descriptions.Item label="Run inputs">
          <Tag>Main</Tag> + <Tag>Memo</Tag> + <Tag>Makefile</Tag> under your current{' '}
          <Tag>Execution</Tag> and <Tag>Output</Tag> settings.
        </Descriptions.Item>
        <Descriptions.Item label="What’s recorded">
          Whatever you enabled in <a href="/help/assignments/config/output">Output</a> (usually
          stdout; optionally stderr and return code).
        </Descriptions.Item>
        <Descriptions.Item label="Storage">
          <b>One text file per task.</b> Each task’s reference output is saved to its own{' '}
          <Tag>.txt</Tag> file when generated.
        </Descriptions.Item>
      </Descriptions>

      <Card className="mt-3">
        <Paragraph className="mb-2">Example (single task, labeled subsections):</Paragraph>
        <CodeEditor
          language="plaintext"
          value={SAMPLE_STDOUT}
          height={160}
          readOnly
          minimal
          fitContent
          showLineNumbers={false}
          hideCopyButton
          title="Memo Output (stdout)"
        />
        <Paragraph className="mt-4 mb-2">Example (multiple tasks via CLI arg + labels):</Paragraph>
        <CodeEditor
          language="plaintext"
          value={SAMPLE_LABELED_TASKS}
          height={220}
          readOnly
          minimal
          fitContent
          showLineNumbers={false}
          hideCopyButton
          title="Memo Output with multiple labeled sections"
        />
      </Card>

      <section id="labels" className="scroll-mt-24" />
      <Title level={3}>Subsections & labels</Title>
      <Paragraph className="mb-0">
        Inside <b>Main</b>, print the delimiter <Text code>&-=-&</Text> followed by the subsection
        name. The text after the delimiter becomes the label used for per-section comparisons and
        reporting. Keep labels stable (e.g., <Text code>task1/Step 1</Text>) so marks line up.
      </Paragraph>

      <section id="regen" className="scroll-mt-24" />
      <Title level={3}>When to regenerate</Title>
      <Table
        className="mt-2"
        size="small"
        columns={regenCols}
        dataSource={regenRows}
        pagination={false}
      />
      <Alert
        className="mt-3"
        type="info"
        showIcon
        message="Regeneration location"
        description="Use the top-right button on the assignment page to regenerate Memo Output."
      />

      <section id="tips" className="scroll-mt-24" />
      <Title level={3}>Tips</Title>
      <ul className="list-disc pl-5">
        <li>
          Keep output <b>deterministic</b> and minimal (avoid timestamps and noisy logs).
        </li>
        <li>
          If you enable <b>stderr</b> or <b>return code</b>, ensure your Main prints or exits
          accordingly.
        </li>
        <li>
          Use clear, consistent <b>labels</b> to align with your mark allocation.
        </li>
      </ul>

      {/* Troubleshooting LAST */}
      <section id="trouble" className="scroll-mt-24" />
      <Title level={3}>Troubleshooting</Title>
      <Collapse
        items={[
          {
            key: 't1',
            label: '“Memo Output not visible for my task”',
            children: (
              <ul className="list-disc pl-5">
                <li>
                  Open <b>Tasks</b>, then pick the exact task and subsection label.
                </li>
                <li>
                  Confirm your Main prints the delimiter <Text code>&-=-&</Text> + label.
                </li>
              </ul>
            ),
          },
          {
            key: 't2',
            label: '“Student results don’t match anymore”',
            children: (
              <Paragraph>
                If you changed <b>Main</b>, <b>Memo</b>, <b>Makefile</b>, <b>Output</b>, the{' '}
                <b>delimiter</b>, or task args, you must <b>regenerate Memo Output</b>.
              </Paragraph>
            ),
          },
          {
            key: 't3',
            label: '“Memo generation timed out / OOM”',
            children: (
              <ul className="list-disc pl-5">
                <li>Trim inputs/logging or optimize the Memo.</li>
                <li>
                  Raise <a href="/help/assignments/config/execution">Execution</a> limits
                  cautiously.
                </li>
              </ul>
            ),
          },
        ]}
      />
    </Space>
  );
}
