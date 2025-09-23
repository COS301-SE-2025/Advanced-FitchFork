// src/pages/help/assignments/TasksHelp.tsx
import { useEffect, useMemo } from 'react';
import { Typography, Card, Alert, Space, Collapse, Table, Descriptions, Tag } from 'antd';
import { useHelpToc } from '@/context/HelpContext';
import { useBreadcrumbContext } from '@/context/BreadcrumbContext';
import { useViewSlot } from '@/context/ViewSlotContext';

const { Title, Paragraph, Text } = Typography;

const toc = [
  { key: 'overview', href: '#overview', title: 'Why Tasks Matter' },
  { key: 'lifecycle', href: '#lifecycle', title: 'How Fitchfork Runs Tasks' },
  { key: 'fields', href: '#fields', title: 'Fields & defaults' },
  { key: 'create', href: '#create', title: 'Create a Task' },
  { key: 'edit', href: '#edit', title: 'Manage Tasks' },
  { key: 'labels', href: '#labels', title: 'Subsections (labels)' },
  { key: 'overwrite', href: '#overwrite', title: 'Overwrite files (per-task)' },
  { key: 'coverage', href: '#coverage', title: 'Code Coverage flag' },
  { key: 'tips', href: '#tips', title: 'Tips' },
  { key: 'trouble', href: '#trouble', title: 'Troubleshooting' },
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
  const { setValue, setBackTo } = useViewSlot();

  useEffect(() => {
    setValue(
      <Typography.Text className="text-base font-medium text-gray-900 dark:text-gray-100 truncate">
        Tasks
      </Typography.Text>,
    );
    setBackTo('/help');
  }, []);

  useEffect(() => {
    setBreadcrumbLabel('help/assignments/tasks', 'Tasks');
  }, []);

  useHelpToc({
    items: toc,
    ids,
    extra: (
      <Card className="mt-4" size="small" title="Quick facts" bordered>
        <ul className="list-disc pl-5">
          <li>Tasks are the runnable units that produce memo and student output for comparison.</li>
          <li>Set them up after uploading Makefile/Main (or Interpreter) and Memo files.</li>
          <li>
            Commands must print labeled sections using your delimiter (default{' '}
            <Text code>&-=-&</Text>).
          </li>
          <li>Each task stores its own memo output text file and optional overwrite archive.</li>
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

      <section id="overview" className="scroll-mt-24" />
      <Title level={3}>Why Tasks Matter</Title>
      <Paragraph className="mb-0">
        Tasks are the execution units that produce both Memo Output and student output. Each task
        shells out to your configured command (often a Make target) and logs labeled sections that
        your Mark Allocator scores. Keeping tasks tidy ensures memo generation, student grading, and
        plagiarism overlays all line up.
      </Paragraph>

      <section id="lifecycle" className="scroll-mt-24" />
      <Title level={3}>How Fitchfork Runs Tasks</Title>
      <Descriptions bordered size="middle" column={1} className="mt-2">
        <Descriptions.Item label="Navigation">
          Open the assignment and select <b>Tasks</b>. Choose a task to view its command,
          subsections, memo output, and any overwrite files.
        </Descriptions.Item>
        <Descriptions.Item label="Pipeline">
          <ul className="list-disc pl-5">
            <li>
              During memo generation, each task command runs with Main + Memo (plus any per-task
              overwrite files) to produce reference output.
            </li>
            <li>
              For student attempts, the same command runs in a fresh workspace with Main + student
              submission + optional overwrite files.
            </li>
            <li>
              Outputs per task are diffed against memo output; subsections map to the labels you
              print in Main.
            </li>
          </ul>
        </Descriptions.Item>
        <Descriptions.Item label="Prerequisites">
          Upload Makefile/Main (or Interpreter) and Memo files first so task commands have
          everything they need.
        </Descriptions.Item>
      </Descriptions>

      <section id="fields" className="scroll-mt-24" />
      <Title level={3}>Fields &amp; defaults</Title>

      {/* Desktop table */}
      <div className="hidden md:block">
        <Table
          size="small"
          columns={fieldCols}
          dataSource={fieldRows}
          pagination={false}
          scroll={{ x: true }}
        />
      </div>

      {/* Mobile card alternative */}
      <div className="block md:hidden !space-y-3">
        {fieldRows.map((r) => (
          <Card
            key={r.key}
            size="small"
            title={<div className="text-base font-semibold truncate">{r.setting}</div>}
          >
            <div className="text-sm text-gray-900 dark:text-gray-100">{r.meaning}</div>
            {r.notes && (
              <div className="mt-2">
                <span className="text-xs uppercase tracking-wide text-gray-500 dark:text-gray-400 mr-2">
                  Notes
                </span>
                <span className="text-sm text-gray-900 dark:text-gray-100">{r.notes}</span>
              </div>
            )}
          </Card>
        ))}
      </div>

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
            or running a compiled binary. If the task has overwrite files, they are merged just
            before this command runs.
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
      <Title level={3}>Overwrite files (per-task)</Title>
      <Paragraph className="mb-0">
        Each task can host an “overlay” archive. When the task command runs, Fitchfork extracts the
        archive into the workspace after the student’s submission and before the command executes.
        Matching paths overwrite the student’s files; new paths are added for that task only.
      </Paragraph>
      <Descriptions bordered size="middle" column={1} className="mt-2">
        <Descriptions.Item label="Why use it?">
          <ul className="list-disc pl-5">
            <li>
              Add fixed inputs, datasets, or golden answers that shouldn’t live in student repos.
            </li>
            <li>Ship small configuration files or scripts to standardize execution per task.</li>
            <li>Patch known issues or provide shim binaries the grader requires.</li>
          </ul>
        </Descriptions.Item>
        <Descriptions.Item label="How it works">
          <ul className="list-disc pl-5">
            <li>
              Upload a <Text code>.zip</Text> under the desired task. Fitchfork stores it in{' '}
              <Text code>overwrite_files/task_X/</Text>.
            </li>
            <li>
              When memo generation or student runs execute the task, the archive is merged after
              extracting Main/Memo or Main/student files.
            </li>
            <li>
              Later uploads replace earlier ones; you can download or delete the current overlay.
            </li>
            <li>Overlays are scoped per task and never leak into other tasks.</li>
          </ul>
        </Descriptions.Item>
        <Descriptions.Item label="Good practice">
          <ul className="list-disc pl-5">
            <li>Keep overlays minimal. Large binaries slow every run.</li>
            <li>Avoid secrets or credentials—student code can read these files.</li>
            <li>Regenerate Memo Output whenever overlays change expected output.</li>
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
