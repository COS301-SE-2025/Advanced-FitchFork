// src/pages/help/assignments/setup/AssignmentSetupHelp.tsx
import { useEffect, useMemo } from 'react';
import { Typography, Card, Alert, Space, Collapse, Table, Descriptions, Tag, Steps } from 'antd';
import { useHelpToc } from '@/context/HelpContext';
import { useBreadcrumbContext } from '@/context/BreadcrumbContext';
import { useViewSlot } from '@/context/ViewSlotContext';

const { Title, Paragraph, Text } = Typography;

const toc = [
  { key: 'overview', href: '#overview', title: 'What “setup” means' },
  { key: 'workflow', href: '#workflow', title: 'Quick Start' },
  { key: 'readiness', href: '#readiness', title: 'Readiness Checklist' },
  { key: 'files', href: '#files', title: 'Files you need' },
  { key: 'tasks', href: '#tasks', title: 'Create Tasks & Subsections' },
  { key: 'memo', href: '#memo', title: 'Generate Memo Output' },
  { key: 'weights', href: '#weights', title: 'Set Mark Allocation' },
  { key: 'ready', href: '#ready', title: 'Ready → Open → Closed' },
  { key: 'tips', href: '#tips', title: 'Tips' },
  { key: 'trouble', href: '#trouble', title: 'Troubleshooting' },
];

type Row = { key: string; item: string; purpose: string; required: string; where: string };

const filesCols = [
  { title: 'Item', dataIndex: 'item', key: 'item', width: 220 },
  { title: 'Purpose', dataIndex: 'purpose', key: 'purpose' },
  { title: 'Required?', dataIndex: 'required', key: 'required', width: 180 },
  { title: 'Where to upload / set', dataIndex: 'where', key: 'where', width: 260 },
];

const filesRows: Row[] = [
  {
    key: 'cfg',
    item: 'Config',
    purpose:
      'Assignment settings: language & mode, execution limits, output capture, marking rules, security, etc.',
    required: 'Yes',
    where: 'Assignments → Config (see: Help → Assignments → Assignment Config → Overview)',
  },
  {
    key: 'mk',
    item: 'Makefile',
    purpose: 'Build/run commands the grader executes (e.g., build targets, run target).',
    required: 'Yes',
    where: 'Assignments → Files → Makefile',
  },
  {
    key: 'main',
    item: 'Main file',
    purpose:
      'Entry point your build/run uses. Prints labeled sections for each subsection using your delimiter (default: &-=-&).',
    required: 'Yes (except GATLAM mode)',
    where: 'Assignments → Files → Main File',
  },
  {
    key: 'interp',
    item: 'Interpreter (GATLAM mode)',
    purpose:
      'Runner that executes candidates and emits outputs for properties/labels. Replaces the need for a Main in GATLAM.',
    required: 'Yes (GATLAM mode)',
    where:
      'Assignments → Files → Memo Files (see: Help → Assignments → Concepts → GATLAM & Interpreter)',
  },
  {
    key: 'memo',
    item: 'Memo files',
    purpose: 'Reference implementation/scripts used to produce the Memo Output.',
    required: 'Yes',
    where: 'Assignments → Files → Memo Files',
  },
  {
    key: 'spec',
    item: 'Specification (starter pack)',
    purpose:
      'Skeleton code for students and base files for plagiarism checks. Must be a flat archive.',
    required: 'Optional (recommended)',
    where: 'Assignments → Files → Specification (files at archive root)',
  },
];

type ReadinessRow = { key: string; item: string; why: string; how: string };

const readinessCols = [
  { title: 'Item', dataIndex: 'item', key: 'item', width: 220 },
  { title: 'Why it matters', dataIndex: 'why', key: 'why' },
  { title: 'How to satisfy', dataIndex: 'how', key: 'how', width: 360 },
];

const readinessRows: ReadinessRow[] = [
  {
    key: 'cfg',
    item: 'Config (config.json)',
    why: 'Defines language, submission mode, marking, execution limits, output, and security.',
    how: 'Assignments → Config. A default exists; review and save to confirm settings.',
  },
  {
    key: 'files',
    item: 'Files',
    why: 'Grader needs a Makefile plus either Main (manual) or Interpreter (GATLAM), and your memo archive.',
    how: 'Assignments → Files. Upload Makefile (+ Main) or Interpreter, and Memo files. Optional: flat Specification.',
  },
  {
    key: 'tasks',
    item: 'Tasks',
    why: 'Each task corresponds to one execution/marking unit and stores outputs.',
    how: 'Assignments → Tasks. Add tasks, then add subsections that match printed labels.',
  },
  {
    key: 'memo',
    item: 'Memo Output',
    why: 'Ground truth used to mark student outputs per task/subsection.',
    how: 'Use “Generate Memo Output” on the assignment page after uploading files.',
  },
  {
    key: 'alloc',
    item: 'Mark Allocator',
    why: 'Defines points per subsection; needed to compute totals.',
    how: 'Assignments → Tasks → pick task → pick subsection → set Marks. Or generate allocator from memo.',
  },
];

export default function AssignmentSetupHelp() {
  const { setBreadcrumbLabel } = useBreadcrumbContext();
  const ids = useMemo(() => toc.map((t) => t.href.slice(1)), []);
  const { setValue, setBackTo } = useViewSlot();

  useEffect(() => {
    setValue(
      <Typography.Text className="text-base font-medium text-gray-900 dark:text-gray-100 truncate">
        Assignment Setup Guide
      </Typography.Text>,
    );
    setBackTo('/help');
  }, []);

  useEffect(() => {
    setBreadcrumbLabel('help/assignments/setup', 'Full Setup Guide');
  }, []);

  useHelpToc({
    items: toc,
    ids,
    extra: (
      <Card className="mt-4" size="small" title="Quick facts" bordered>
        <ul className="list-disc pl-5">
          <li>
            Status flow: <b>Setup → Ready → Open → Closed</b>. The Setup panel tells you what’s
            missing.
          </li>
          <li>
            Ready means you’ve saved Config, uploaded required files, created Tasks/Subsections,
            generated Memo Output, and set Mark Allocation.
          </li>
          <li>
            Manual submissions need a <b>Main</b>; <Tag color="purple">gatlam</Tag> needs an{' '}
            <b>Interpreter</b> instead.
          </li>
          <li>
            Specification ZIPs must be flat (no nested folders); memo generation and MOSS rely on
            that layout.
          </li>
        </ul>
      </Card>
    ),
    onMountScrollToHash: true,
  });

  return (
    <Space direction="vertical" size="small" className="w-full !p-0">
      <Title level={2} className="mb-0">
        Full Setup Guide
      </Title>

      <section id="overview" className="scroll-mt-24" />
      <Title level={3}>What “setup” means</Title>
      <Paragraph className="mb-0">
        “Setup” is the stage where you assemble everything the grader needs: configuration,
        build/run files, memo references, tasks, and marks. When each piece is present and memo
        output has been generated, the status flips to
        <b>Ready</b>; from there the assignment can open and close on schedule without further
        intervention.
      </Paragraph>

      <section id="workflow" className="scroll-mt-24" />
      <Title level={3}>Quick Start</Title>
      <Steps
        direction="vertical"
        items={[
          {
            title: 'Configure the assignment',
            description:
              'Review config.json (Language & Mode, Execution, Output, Marking, Security) so the runner knows how to build and compare.',
          },
          {
            title: 'Upload files',
            description:
              'Provide Makefile, Main (or Interpreter for Gatlam), Memo files, and optionally a flat Specification archive for students.',
          },
          {
            title: 'Create Tasks & Subsections',
            description:
              'Add tasks and their subsections so each command produces labeled blocks that align with your outputs.',
          },
          {
            title: 'Generate Memo Output',
            description:
              'Use the top-right button to capture reference output per task. The Setup panel will stay red until this succeeds.',
          },
          {
            title: 'Set Mark Allocation',
            description:
              'Assign points to each subsection (Tasks → subsection → Marks) so scoring matches your rubric.',
          },
          {
            title: 'Schedule & status',
            description:
              'Set Available From/Due Date. Once the checklist is green, change status to Ready and let the schedule open it automatically.',
          },
        ]}
      />

      <section id="readiness" className="scroll-mt-24" />
      <Title level={3}>Readiness Checklist</Title>
      <Paragraph className="mb-2">
        The “Setup incomplete” panel in the assignment layout checks these exact items. Use the list
        below as a quick map to the problem: if the panel highlights something in red, fix the
        matching row here and re-run Memo Output if necessary.
      </Paragraph>
      {/* md+ : normal table */}
      <div className="hidden md:block">
        <Table
          size="small"
          columns={readinessCols}
          dataSource={readinessRows}
          pagination={false}
          scroll={{ x: true }}
        />
      </div>

      {/* <md : cards (no extra shadows) */}
      <div className="block md:hidden !space-y-3">
        {readinessRows.map((r) => (
          <Card
            key={r.key}
            size="small"
            title={<div className="text-base font-semibold truncate">{r.item}</div>}
          >
            <div className="text-xs uppercase tracking-wide text-gray-500 dark:text-gray-400 mb-1">
              Why it matters
            </div>
            <div className="text-sm text-gray-900 dark:text-gray-100 mb-2">{r.why}</div>

            <div className="text-xs uppercase tracking-wide text-gray-500 dark:text-gray-400 mb-1">
              How to satisfy
            </div>
            <div className="text-sm text-gray-900 dark:text-gray-100">{r.how}</div>
          </Card>
        ))}
      </div>

      <Alert
        className="mt-2"
        type="info"
        showIcon
        message="Submission modes and file requirements"
        description={
          <>
            <div className="mb-1">
              <Tag>manual</Tag>: requires <b>Main</b>. <Tag>gatlam</Tag>: requires{' '}
              <b>Interpreter</b> (no Main required).
            </div>
            <div>
              Other modes like <Tag>rng</Tag> or <Tag>codecoverage</Tag> don’t require
              Main/Interpreter.
            </div>
          </>
        }
      />

      <section id="files" className="scroll-mt-24" />
      <Title level={3}>Files you need</Title>
      <Paragraph className="mb-2">
        Upload these under <b>Assignments → Files</b>. Manual mode needs Makefile + Main + Memo;
        GATLAM replaces Main with an Interpreter. The Specification archive is optional but strongly
        recommended as the starter pack and plagiarism base files.
      </Paragraph>
      {/* md+ : normal table */}
      <div className="hidden md:block">
        <Table
          size="small"
          columns={filesCols}
          dataSource={filesRows}
          pagination={false}
          scroll={{ x: true }}
        />
      </div>

      {/* <md : cards (no extra shadows) */}
      <div className="block md:hidden !space-y-3">
        {filesRows.map((r) => (
          <Card
            key={r.key}
            size="small"
            title={<div className="text-base font-semibold truncate">{r.item}</div>}
          >
            <div className="text-xs uppercase tracking-wide text-gray-500 dark:text-gray-400 mb-1">
              Purpose
            </div>
            <div className="text-sm text-gray-900 dark:text-gray-100 mb-2">{r.purpose}</div>

            <div className="text-xs uppercase tracking-wide text-gray-500 dark:text-gray-400 mb-1">
              Required?
            </div>
            <div className="text-sm text-gray-900 dark:text-gray-100 mb-2">{r.required}</div>

            <div className="text-xs uppercase tracking-wide text-gray-500 dark:text-gray-400 mb-1">
              Where to upload / set
            </div>
            <div className="text-sm text-gray-900 dark:text-gray-100">{r.where}</div>
          </Card>
        ))}
      </div>

      <Alert
        className="mt-2"
        type="info"
        showIcon
        message="Labels and the delimiter"
        description={
          <>
            Subsections are matched by labels printed in your program. By default, print the
            delimiter <Text code>&-=-&</Text> followed by the label name on its own line, then the
            lines that belong to that label. See{' '}
            <a href="/help/assignments/memo-output">Memo Output</a> for how labels are compared.
          </>
        }
      />
      <Card className="mt-3" bordered>
        <Paragraph className="mb-0">
          Specification archives must be flat. Place all files at the root of the archive. Do not
          include folders or nested directories. See{' '}
          <a href="/help/assignments/files/specification">Specification</a>.
        </Paragraph>
      </Card>

      <section id="tasks" className="scroll-mt-24" />
      <Title level={3}>Create Tasks & Subsections</Title>
      <ul className="list-disc pl-5">
        <li>
          Open <b>Tasks</b> and add each task for the assignment.
        </li>
        <li>
          Within a task, add <b>Subsections</b>. Each subsection corresponds to a label your program
          prints.
        </li>
        <li>
          Names should match the labels printed by your Main (or Interpreter in GATLAM) using{' '}
          <Text code>&-=-&</Text>.
        </li>
        <li>
          Need per-task fixtures? Upload an <b>overwrite</b> archive on the task; those files are
          merged before the command runs.
        </li>
        <li>
          You can refine subsections later, but renaming labels creates new buckets and may require
          regenerating Memo Output.
        </li>
      </ul>

      <section id="memo" className="scroll-mt-24" />
      <Title level={3}>Generate Memo Output</Title>
      <Descriptions bordered size="middle" column={1} className="mt-2">
        <Descriptions.Item label="Where">
          Open the assignment and use the <b>Generate Memo Output</b> button in the top-right.
        </Descriptions.Item>
        <Descriptions.Item label="What happens">
          The platform builds and runs your <Tag>Memo files</Tag> (or Interpreter in GATLAM) to
          produce the expected output for each task. Each task’s memo output is saved as a separate
          text file alongside the assignment.
        </Descriptions.Item>
        <Descriptions.Item label="How to view">
          Go to <b>Tasks</b> → choose a <b>task</b> → choose a <b>subsection</b> to view the memo
          lines for that label.
        </Descriptions.Item>
        <Descriptions.Item label="Self-check">
          After generating, open a few tasks to confirm outputs look correct before opening the
          assignment to students.
        </Descriptions.Item>
      </Descriptions>

      <section id="weights" className="scroll-mt-24" />
      <Title level={3}>Set Mark Allocation</Title>
      <Paragraph className="mb-0">
        Weights live at the subsection (label) level and roll up to task totals. To edit: go to{' '}
        <b>Tasks</b> → pick the <b>task</b> → pick a <b>subsection</b>, then set the <b>Marks</b>.
        You can also generate the allocator from memo output to seed default weights. See
        <a href="/help/assignments/mark-allocator">Mark Allocation</a> for examples.
      </Paragraph>

      <section id="ready" className="scroll-mt-24" />
      <Title level={3}>Ready → Open → Closed</Title>
      <Descriptions bordered size="middle" column={1} className="mt-2">
        <Descriptions.Item label="Readiness checklist">
          The assignment becomes <b>Ready</b> when all of these are present:
          <ul className="list-disc pl-5 mt-1">
            <li>Config (saved)</li>
            <li>
              Files uploaded: Makefile, Main <i>(manual)</i> or Interpreter <i>(gatlam)</i>, Memo
              files
            </li>
            <li>Tasks & Subsections (labels match program output)</li>
            <li>Memo Output (generated for all tasks)</li>
            <li>Mark Allocator (points set per subsection)</li>
          </ul>
        </Descriptions.Item>
        <Descriptions.Item label="Open/Close schedule">
          Set <b>Available From</b> and <b>Due Date</b>.
          <ul className="list-disc pl-5 mt-1">
            <li>
              <Tag color="gold">Setup</Tag>: one or more checklist items are missing.
            </li>
            <li>
              <Tag color="green">Ready</Tag>: checklist complete; waiting to open.
            </li>
            <li>
              <Tag color="blue">Open</Tag>: students can submit between the availability window.
            </li>
            <li>
              <Tag color="red">Closed</Tag>: due date passed; submission button disabled.
            </li>
          </ul>
          The system transitions Ready → Open → Closed automatically based on dates; it never jumps
          directly from Ready to Closed.
        </Descriptions.Item>
      </Descriptions>

      <section id="tips" className="scroll-mt-24" />
      <Title level={3}>Tips</Title>
      <ul className="list-disc pl-5">
        <li>
          Start with default execution limits; only raise CPU/time/memory if a task consistently
          needs it.
        </li>
        <li>
          Capture only the streams you compare in{' '}
          <a href="/help/assignments/config/output">Output settings</a> to keep diffs tidy.
        </li>
        <li>
          Pick a marking comparator (
          <a href="/help/assignments/config/marking">Exact, Percentage, Regex</a>) that matches your
          memo output style.
        </li>
        <li>
          Keep subsection labels stable — renaming them creates new buckets and usually means
          regenerating Memo Output.
        </li>
        <li>
          Ensure Specification ZIPs and student submissions remain flat archives (no folders); the
          grader expects that shape.
        </li>
        <li>
          If you’re using GATLAM, review <a href="/help/assignments/gatlam">GATLAM & Interpreter</a>{' '}
          and
          <a href="/help/assignments/config/gatlam">GATLAM Config</a> so the generated programs
          print the labels you need.
        </li>
        <li>
          When Tasks auto-detect targets from the Makefile, double-check the generated commands
          before relying on them.
        </li>
      </ul>

      {/* Troubleshooting LAST */}
      <section id="trouble" className="scroll-mt-24" />
      <Title level={3}>Troubleshooting</Title>
      <Collapse
        items={[
          {
            key: 't1',
            label: '“Why is my assignment still in Setup?”',
            children: (
              <ul className="list-disc pl-5">
                <li>
                  Open the Setup panel and verify each checklist item: Config saved, required files
                  uploaded, tasks/subsections defined, memo output generated, marks set.
                </li>
                <li>
                  In GATLAM mode, ensure the <b>Interpreter</b> is uploaded (Main is not required).
                </li>
                <li>
                  If using <b>rng</b> or <b>codecoverage</b> modes, Main/Interpreter are not
                  required — focus on other items.
                </li>
              </ul>
            ),
          },
          {
            key: 't2',
            label: '“Memo Output is empty or missing labels”',
            children: (
              <ul className="list-disc pl-5">
                <li>
                  Confirm your Main (or Interpreter) prints labels using the delimiter{' '}
                  <Text code>&-=-&</Text> and that each label appears at least once in memo output.
                </li>
                <li>
                  Re-run <b>Generate Memo Output</b> after changing code, labels, or config.
                </li>
                <li>
                  Ensure the task commands actually produce the expected stdout/stderr captured in
                  config.
                </li>
              </ul>
            ),
          },
          {
            key: 't3',
            label: '“Specification upload rejected”',
            children: (
              <ul className="list-disc pl-5">
                <li>
                  Use a supported archive and place all files at the root (no nested directories).
                </li>
                <li>Do not include solutions or private tests in the starter pack.</li>
              </ul>
            ),
          },
          {
            key: 't4',
            label: '“Marks aren’t adding up”',
            children: (
              <ul className="list-disc pl-5">
                <li>
                  Open <b>Tasks</b> → select the task → verify each subsection’s marks. Task total
                  is their sum.
                </li>
                <li>Assignment total is the sum of all task totals.</li>
                <li>
                  Coverage-only tasks don’t contribute to allocator points (visible in reports
                  only).
                </li>
              </ul>
            ),
          },
          {
            key: 't5',
            label: '“Students can’t submit”',
            children: (
              <ul className="list-disc pl-5">
                <li>
                  Ensure the assignment is <b>Open</b> (check dates), and any security requirements
                  (PIN, IP allowlist) are satisfied.
                </li>
                <li>
                  If attempt limits are enabled, verify remaining attempts in the assignment’s
                  policy.
                </li>
              </ul>
            ),
          },
        ]}
      />
    </Space>
  );
}
