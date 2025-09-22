// src/pages/help/assignments/TasksHelp.tsx
import { useEffect, useMemo } from 'react';
import { Typography, Card, Alert, Space, Collapse, Table, Descriptions, Tag } from 'antd';
import { useHelpToc } from '@/context/HelpContext';
import { useBreadcrumbContext } from '@/context/BreadcrumbContext';

const { Title, Paragraph, Text } = Typography;

const toc = [
  { key: 'what', href: '#what', title: 'What are Tasks?' },
  { key: 'where', href: '#where', title: 'Where to find Tasks' },
  { key: 'fields', href: '#fields', title: 'Fields & defaults' },
  { key: 'create', href: '#create', title: 'Create a Task' },
  { key: 'edit', href: '#edit', title: 'Edit, reorder, delete' },
  { key: 'labels', href: '#labels', title: 'Subsections (labels)' },
  { key: 'overwrite', href: '#overwrite', title: 'Overwrite files (per‑task)' },
  { key: 'coverage', href: '#coverage', title: 'Code Coverage flag' },
  { key: 'tips', href: '#tips', title: 'Tips' },
  { key: 'trouble', href: '#trouble', title: 'Troubleshooting' }, // keep last
];

// UI table for task fields
const fieldCols = [
  { title: 'Setting', dataIndex: 'setting', key: 'setting', width: 220 },
  { title: 'What it does', dataIndex: 'meaning', key: 'meaning' },
  { title: 'Notes', dataIndex: 'notes', key: 'notes', width: 320 },
];

const fieldRows = [
  {
    key: 'num',
    setting: 'Task number',
    meaning: 'Display order of tasks (shown as “Task 1”, “Task 2”, …).',
    notes: 'Use integers starting at 1. Lower numbers appear first.',
  },
  {
    key: 'name',
    setting: 'Name',
    meaning: 'Human-readable title for the task.',
    notes: 'Keep it short but specific; used throughout the UI and reports.',
  },
  {
    key: 'command',
    setting: 'Command',
    meaning:
      'The command the grader executes for this task (often a make target or binary). Must print your labeled output.',
    notes:
      'Non-interactive; respect Execution limits; exit 0 on success. Prefer deterministic output.',
  },
  {
    key: 'cc',
    setting: 'Code Coverage',
    meaning: 'Toggle whether this task contributes coverage marks.',
    notes:
      'Requires coverage config and instrumentation. See Help → Assignments → Concepts → Code Coverage.',
  },
];

export default function TasksHelp() {
  const { setBreadcrumbLabel } = useBreadcrumbContext();
  const ids = useMemo(() => toc.map((t) => t.href.slice(1)), []);

  useEffect(() => {
    setBreadcrumbLabel('help/assignments/tasks', 'Tasks');
  }, []);

  useHelpToc({
    items: toc,
    ids,
    extra: (
      <Card className="mt-4" size="small" title="Quick facts" bordered>
        <ul className="list-disc pl-5">
          <li>A Task is a single runnable check in an assignment.</li>
          <li>
            Define Tasks after uploading your Makefile/Main (or Interpreter in GATLAM) and Memo
            files.
          </li>
          <li>
            Outputs must include labeled sections using your delimiter (default{' '}
            <Text code>&-=-&</Text>).
          </li>
          <li>
            Memo Output is generated per task; each task’s memo is saved as its own text file.
          </li>
        </ul>
      </Card>
    ),
    onMountScrollToHash: true,
  });

  return (
    <Space direction="vertical" size="small" className="w-full !p-0">
      <Title level={2} className="mb-0">
        Tasks
      </Title>

      <section id="what" className="scroll-mt-24" />
      <Title level={3}>What are Tasks?</Title>
      <Paragraph className="mb-0">
        Tasks split an assignment into independently runnable checks. Each task runs a configured{' '}
        <b>command</b>, captures output, and is graded against your{' '}
        <a href="/help/assignments/memo-output">Memo Output</a> and{' '}
        <a href="/help/assignments/config/marking">Marking</a> rules. Tasks also host their{' '}
        <b>subsections</b> (labels) where you allocate{' '}
        <a href="/help/assignments/mark-allocator">Marks</a>.
      </Paragraph>

      <section id="where" className="scroll-mt-24" />
      <Title level={3}>Where to find Tasks</Title>
      <Descriptions bordered size="middle" column={1} className="mt-2">
        <Descriptions.Item label="Navigation">
          Open the assignment, then go to the <b>Tasks</b> page. Pick a task to see its subsections
          and memo output per label.
        </Descriptions.Item>
        <Descriptions.Item label="Prerequisites">
          Upload files first (Makefile, Main or Interpreter for GATLAM, Memo files). See the{' '}
          <a href="/help/assignments/setup">Full Setup Guide</a>.
        </Descriptions.Item>
      </Descriptions>

      <section id="fields" className="scroll-mt-24" />
      <Title level={3}>Fields & defaults</Title>
      <Table
        className="mt-2"
        size="small"
        columns={fieldCols}
        dataSource={fieldRows}
        pagination={false}
      />
      <Alert
        className="mt-2"
        type="info"
        showIcon
        message="About the command"
        description={
          <>
            The command should run non-interactively, finish within your{' '}
            <a href="/help/assignments/config/execution">Execution</a> limits, and print
            deterministic, labeled output blocks. Common patterns are <Text code>make task1</Text>{' '}
            or running a compiled binary.
          </>
        }
      />

      <section id="create" className="scroll-mt-24" />
      <Title level={3}>Create a Task</Title>
      <ol className="list-decimal pl-5">
        <li>
          Ensure required files are uploaded: <b>Makefile</b> and <b>Main</b> (or <b>Interpreter</b>{' '}
          in GATLAM), plus <b>Memo files</b>.
        </li>
        <li>
          Open the assignment’s <b>Tasks</b> page and choose <b>Add Task</b>.
        </li>
        <li>
          Fill in <b>Task number</b>, <b>Name</b>, and <b>Command</b>. Toggle <b>Code Coverage</b>{' '}
          if applicable.
        </li>
        <li>
          Save the task, then add its <b>Subsections</b> (labels) under the task.
        </li>
        <li>
          Generate <b>Memo Output</b> from the assignment page (top-right button).
        </li>
      </ol>

      <section id="edit" className="scroll-mt-24" />
      <Title level={3}>Edit, reorder, delete</Title>
      <ul className="list-disc pl-5">
        <li>
          <b>Edit</b> name, command, or Code Coverage from the task’s details page.
        </li>
        <li>
          <b>Reorder</b> tasks by adjusting <b>Task number</b> (lower numbers appear first).
        </li>
        <li>
          <b>Delete</b> removes the task configuration. Review your{' '}
          <a href="/help/assignments/memo-output">Memo Output</a> and{' '}
          <a href="/help/assignments/mark-allocator">Mark Allocation</a> afterwards to keep things
          consistent.
        </li>
      </ul>

  <section id="labels" className="scroll-mt-24" />
  <Title level={3}>Subsections (labels)</Title>
      <Paragraph className="mb-0">
        Inside a task, create subsections that mirror the labels your program prints. In your{' '}
        <b>Main</b> (or <b>Interpreter</b> for GATLAM), print the delimiter <Text code>&-=-&</Text>{' '}
        followed by the subsection name on its own line, then the lines that belong to that label.
        These labeled blocks are what get compared to the task’s Memo Output. See{' '}
        <a href="/help/assignments/memo-output">Memo Output</a> for examples and{' '}
        <a href="/help/assignments/mark-allocator">Mark Allocation</a> to assign marks per label.
  </Paragraph>

  <section id="overwrite" className="scroll-mt-24" />
  <Title level={3}>Overwrite files (per‑task)</Title>
  <Paragraph className="mb-0">
    For each task you can upload a small archive of files that is applied on top of the student’s
    submission when that task runs. Think of it as a temporary “overlay”: files with the same
    paths replace the student’s versions; new files are added for that run only.
  </Paragraph>
  <Descriptions bordered size="middle" column={1} className="mt-2">
    <Descriptions.Item label="Why use it?">
      <ul className="list-disc pl-5">
        <li>Add fixed inputs, test data, or golden resources the task needs.</li>
        <li>Pin a config file or small script so all students run with known settings.</li>
        <li>Provide stubs/wrappers for tools not included in student submissions.</li>
      </ul>
    </Descriptions.Item>
    <Descriptions.Item label="How it works">
      <ul className="list-disc pl-5">
        <li>Upload a <Text code>.zip</Text> under the specific task.</li>
        <li>The contents are merged at the project root for that task’s execution.</li>
        <li>Matching paths overwrite the student’s files; new paths are added.</li>
        <li>The newest upload is used. You can download or delete the current overlay.</li>
        <li>Overlays are per‑task and do not affect other tasks.</li>
      </ul>
    </Descriptions.Item>
    <Descriptions.Item label="Good practice">
      <ul className="list-disc pl-5">
        <li>Keep overlays small and only what’s necessary for the task.</li>
        <li>Do not include secrets or credentials — students’ code can read these files.</li>
        <li>Regenerate Memo Output if the overlay changes expected output.</li>
      </ul>
    </Descriptions.Item>
  </Descriptions>

      <section id="coverage" className="scroll-mt-24" />
      <Title level={3}>Code Coverage flag</Title>
      <Descriptions bordered size="middle" column={1} className="mt-2">
        <Descriptions.Item label="When to enable">
          Turn on <Tag color="geekblue">Code Coverage</Tag> if this task’s runs should contribute to
          the assignment’s coverage target.
        </Descriptions.Item>
        <Descriptions.Item label="Requirements">
          Coverage tools and config must be set up in your build/run. See{' '}
          <a href="/help/assignments/code-coverage">Code Coverage</a>.
        </Descriptions.Item>
      </Descriptions>

      <section id="tips" className="scroll-mt-24" />
      <Title level={3}>Tips</Title>
      <ul className="list-disc pl-5">
        <li>
          Keep commands short and deterministic. Avoid network calls or random output unless
          required.
        </li>
        <li>
          Fail fast with clear exit codes; use stderr for genuine errors and stdout for graded
          output.
        </li>
        <li>Stabilize label names early to avoid creating new subsections unintentionally.</li>
        <li>
          After changing commands or labels, regenerate{' '}
          <a href="/help/assignments/memo-output">Memo Output</a>.
        </li>
      </ul>

      {/* Troubleshooting LAST */}
      <section id="trouble" className="scroll-mt-24" />
      <Title level={3}>Troubleshooting</Title>
      <Collapse
        items={[
          {
            key: 't1',
            label: '“My task runs locally but fails on the grader”',
            children: (
              <ul className="list-disc pl-5">
                <li>
                  Check <a href="/help/assignments/config/execution">Execution limits</a> (time,
                  memory, processes).
                </li>
                <li>
                  Ensure the command is non-interactive and has all inputs bundled with the
                  assignment.
                </li>
              </ul>
            ),
          },
          {
            key: 't2',
            label: '“No subsections are showing under the task”',
            children: (
              <ul className="list-disc pl-5">
                <li>
                  Verify your program prints labels with <Text code>&-=-&</Text> and the subsection
                  name.
                </li>
                <li>
                  Generate or regenerate <b>Memo Output</b> so the system learns the labels.
                </li>
              </ul>
            ),
          },
          {
            key: 't3',
            label: '“Unexpected differences vs. Memo Output”',
            children: (
              <ul className="list-disc pl-5">
                <li>
                  Trim extra logs; keep output deterministic and consistent with line endings.
                </li>
                <li>
                  Confirm <a href="/help/assignments/config/output">Output capture</a> options
                  include the streams you compare (stdout/stderr/retcode).
                </li>
              </ul>
            ),
          },
          {
            key: 't4',
            label: '“Coverage shows zero for this task”',
            children: (
              <ul className="list-disc pl-5">
                <li>
                  Enable the <b>Code Coverage</b> toggle on the task.
                </li>
                <li>Verify coverage tooling and instrumentation in your build commands.</li>
              </ul>
            ),
          },
        ]}
      />
    </Space>
  );
}
