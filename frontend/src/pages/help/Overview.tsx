// src/pages/help/getting-started/Overview.tsx
import React, { useEffect, useMemo } from 'react';
import { Typography, Card, Space, Alert, Descriptions, Tag, Steps, Timeline, Table } from 'antd';
import {
  RocketOutlined,
  SettingOutlined,
  FileZipOutlined,
  DiffOutlined,
  CheckCircleOutlined,
  SafetyOutlined,
  UploadOutlined,
  BranchesOutlined,
} from '@ant-design/icons';
import { useHelpToc } from '@/context/HelpContext';
import { useBreadcrumbContext } from '@/context/BreadcrumbContext';
import { useViewSlot } from '@/context/ViewSlotContext';
import { GatlamLink } from '@/components/common';

const { Title, Paragraph, Text } = Typography;

const toc = [
  { key: 'what', href: '#what', title: 'What is Fitchfork?' },
  { key: 'who', href: '#who', title: 'Who uses it' },
  { key: 'concepts', href: '#concepts', title: 'Core concepts' },
  { key: 'flows', href: '#flows', title: 'Common workflows' },
  { key: 'grading', href: '#grading', title: 'How grading works' },
  { key: 'modes', href: '#modes', title: 'Submission modes' },
  { key: 'roles', href: '#roles', title: 'Roles & permissions' },
  { key: 'links', href: '#links', title: 'Where to next?' },
];

type Row = { key: string; area: string; what: React.ReactNode; link?: React.JSX.Element };

const conceptCols = [
  { title: 'Area', dataIndex: 'area', key: 'area', width: 220 },
  { title: 'What it is', dataIndex: 'what', key: 'what' },
  { title: 'Learn more', dataIndex: 'link', key: 'link', width: 260 },
];

const conceptRows: Row[] = [
  {
    key: 'assignments',
    area: 'Assignments',
    what: 'Containers for everything: config, files, tasks, memo output, marking, submissions, and plagiarism.',
    link: <a href="/help/assignments/setup">Full Setup Guide →</a>,
  },
  {
    key: 'config',
    area: 'Assignment Config',
    what: (
      <>
        Language &amp; Mode, Execution limits, Output capture, Marking rules, Security,{' '}
        <GatlamLink tone="inherit" icon={false} underline={false}>
          GATLAM
        </GatlamLink>
        , Code Coverage.
      </>
    ),
    link: <a href="/help/assignments/config">Config Overview →</a>,
  },
  {
    key: 'files',
    area: 'Files',
    what: 'Makefile + Main (or Interpreter) + Memo archives; optional Specification (starter pack & MOSS base).',
    link: <a href="/help/assignments/files/makefile">Makefile / Main / Memo →</a>,
  },
  {
    key: 'tasks',
    area: 'Tasks & Subsections',
    what: 'Runnable units that produce outputs. Subsections map to labeled blocks printed by your code.',
    link: <a href="/help/assignments/tasks">Tasks →</a>,
  },
  {
    key: 'memo',
    area: 'Memo Output',
    what: 'Reference outputs generated from your memo code. Student outputs are compared against these.',
    link: <a href="/help/assignments/memo-output">Memo Output →</a>,
  },
  {
    key: 'allocator',
    area: 'Mark Allocation',
    what: 'Points per subsection; totals roll up per task and per assignment.',
    link: <a href="/help/assignments/mark-allocator">Mark Allocation →</a>,
  },
  {
    key: 'moss',
    area: 'Plagiarism (MOSS)',
    what: 'Automates MOSS runs, creates cases, and mirrors reports for offline review.',
    link: <a href="/help/assignments/plagiarism/moss">Plagiarism & MOSS →</a>,
  },
];

const modeCols = [
  { title: 'Mode', dataIndex: 'mode', key: 'mode', width: 180 },
  { title: 'What it does', dataIndex: 'desc', key: 'desc' },
  { title: 'Requires', dataIndex: 'req', key: 'req', width: 260 },
];

const modeRows = [
  {
    key: 'manual',
    mode: 'manual',
    desc: 'Runs your Makefile + Main against memo and student submissions.',
    req: 'Makefile + Main + Memo',
  },
  {
    key: 'gatlam',
    mode: (
      <GatlamLink tone="inherit" icon={false} underline={false}>
        gatlam
      </GatlamLink>
    ),
    desc: 'Genetic-assisted runs via an Interpreter; can regenerate allocator/memo as needed.',
    req: 'Interpreter (+ Memo recommended)',
  },
  {
    key: 'rng',
    mode: 'rng',
    desc: 'Experimental randomised runs for exploration.',
    req: 'Depends on your pipeline',
  },
  {
    key: 'coverage',
    mode: 'codecoverage',
    desc: 'Enforces coverage threshold; coverage tasks don’t contribute points.',
    req: 'Coverage tooling + config',
  },
];

const roleCols = [
  { title: 'Role', dataIndex: 'role', key: 'role', width: 200 },
  { title: 'Can', dataIndex: 'can', key: 'can' },
];

const roleRows = [
  {
    key: 'lecturer',
    role: 'lecturer',
    can: 'Create/edit assignments, upload files, configure marking/security, run memo & MOSS, view all results.',
  },
  {
    key: 'assistant',
    role: 'assistant_lecturer',
    can: 'Most lecturer capabilities for day-to-day setup and marking.',
  },
  {
    key: 'student',
    role: 'student',
    can: 'Submit within the open window, view results/feedback, agree to ownership attestations.',
  },
];

export default function FitchforkOverview() {
  const { setBreadcrumbLabel } = useBreadcrumbContext();
  const ids = useMemo(() => toc.map((t) => t.href.slice(1)), []);
  const { setValue, setBackTo } = useViewSlot();

  useEffect(() => {
    setValue(
      <Typography.Text className="text-base font-medium text-gray-900 dark:text-gray-100 truncate">
        Fitchfork Overview
      </Typography.Text>,
    );
    setBackTo('/help');
  }, []);

  useEffect(() => {
    // adjust the key/label to whatever you use in HELP_MENU_ITEMS for this page
    setBreadcrumbLabel('help/getting-started/overview', 'Overview');
  }, []);

  useHelpToc({
    items: toc,
    ids,
    extra: (
      <Card className="mt-4" size="small" title="At a glance" bordered>
        <ul className="list-disc pl-5">
          <li>
            Status flow: <Tag color="gold">Setup</Tag> → <Tag color="green">Ready</Tag> →{' '}
            <Tag color="blue">Open</Tag> → <Tag color="red">Closed</Tag>.
          </li>
          <li>
            Compare <b>student output</b> vs <b>Memo Output</b> by labeled subsections (
            <Text code>&amp;-=-&amp;</Text>).
          </li>
          <li>
            Files live in separate archives: <b>Makefile</b>, <b>Main</b> (or <b>Interpreter</b>),
            <b> Memo</b>, optional <b>Specification</b>.
          </li>
        </ul>
      </Card>
    ),
    onMountScrollToHash: true,
  });

  return (
    <Space direction="vertical" size="small" className="w-full !p-0">
      <Title level={2} className="mb-0">
        Fitchfork Overview
      </Title>

      <section id="what" className="scroll-mt-24" />
      <Title level={3}>What is Fitchfork?</Title>
      <Paragraph className="mb-0">
        <strong>Fitchfork</strong> is a programming assignment platform that builds, runs, and marks
        student submissions against your <b>reference outputs (Memo Output)</b>. You define tasks,
        capture outputs, assign marks per labeled subsection, and the platform handles submission
        windows, resubmissions, plagiarism checks (MOSS), and rich feedback.
      </Paragraph>

      <section id="who" className="scroll-mt-24" />
      <Title level={3}>Who uses it</Title>
      <Descriptions bordered size="middle" column={1} className="mt-2">
        <Descriptions.Item label="Lecturers & assistants">
          Configure assignments, upload archives, define tasks/marks, generate memo, run MOSS.
        </Descriptions.Item>
        <Descriptions.Item label="Students">
          Submit a single archive during the open window; results and feedback appear per attempt.
        </Descriptions.Item>
      </Descriptions>

      <section id="concepts" className="scroll-mt-24" />
      <Title level={3}>Core concepts</Title>
      <div className="hidden md:block">
        <Table
          size="small"
          columns={conceptCols}
          dataSource={conceptRows}
          pagination={false}
          scroll={{ x: true }}
        />
      </div>

      <div className="block md:hidden mt-2 !space-y-3">
        {conceptRows.map((r) => (
          <Card
            key={r.key}
            size="small"
            title={<div className="text-base font-semibold truncate">{r.area}</div>}
          >
            <div className="text-sm text-gray-900 dark:text-gray-100">{r.what}</div>
            {r.link && (
              <div className="mt-2">
                <span className="text-xs uppercase tracking-wide text-gray-500 dark:text-gray-400 mr-2">
                  Learn more
                </span>
                {r.link}
              </div>
            )}
          </Card>
        ))}
      </div>

      <section id="flows" className="scroll-mt-24" />
      <Title level={3}>Common workflows</Title>
      <Steps
        direction="vertical"
        items={[
          {
            title: 'Set up the assignment',
            icon: <SettingOutlined />,
            description:
              'Pick Language & Mode, review Execution/Output/Marking/Security, and save config.',
          },
          {
            title: 'Upload files',
            icon: <FileZipOutlined />,
            description:
              'Upload Makefile + Main (or Interpreter) + Memo; optional Specification for starter code & MOSS base.',
          },
          {
            title: 'Create Tasks & Subsections',
            icon: <BranchesOutlined />,
            description:
              'Define commands per task and print labeled blocks in code using the delimiter (###).',
          },
          {
            title: 'Generate Memo Output',
            icon: <RocketOutlined />,
            description: 'Capture reference outputs; set Mark Allocation per subsection.',
          },
          {
            title: 'Open for submissions',
            icon: <UploadOutlined />,
            description:
              'Status moves Ready → Open on schedule. Students submit archives; runs are queued and marked.',
          },
          {
            title: 'Review & integrity',
            icon: <DiffOutlined />,
            description: 'Inspect results, run MOSS, and resolve plagiarism cases as needed.',
          },
          {
            title: 'Close & follow-up',
            icon: <CheckCircleOutlined />,
            description: 'Close on due date; optionally remark/resubmit for maintenance.',
          },
        ]}
      />

      <section id="grading" className="scroll-mt-24" />
      <Title level={3}>How grading works (high level)</Title>
      <Timeline
        className="mb-2"
        items={[
          {
            color: 'blue',
            dot: <RocketOutlined />,
            children:
              'Memo generation runs your Makefile + Main (or Interpreter) and saves memo text per task.',
          },
          {
            color: 'green',
            dot: <DiffOutlined />,
            children:
              'Each student run executes the same command; output is compared to Memo Output by subsection.',
          },
          {
            color: 'gray',
            dot: <SafetyOutlined />,
            children:
              'Mark Allocation assigns points; disallowed code rules and coverage settings apply if configured.',
          },
        ]}
      />
      <Alert
        type="info"
        showIcon
        message="Labels drive marking"
        description={
          <>
            Print <Text code>###</Text> followed by the subsection name on its own line to start
            each labeled block. Marks are assigned per subsection; totals roll up automatically.
          </>
        }
      />

      <section id="modes" className="scroll-mt-24" />
      <Title level={3}>Submission modes</Title>

      {/* md+ : normal table */}
      <div className="hidden md:block">
        <Table
          size="small"
          columns={modeCols}
          dataSource={modeRows}
          pagination={false}
          scroll={{ x: true }}
        />
      </div>

      {/* <md : cards (no extra shadows) */}
      <div className="block md:hidden mt-2 !space-y-3">
        {modeRows.map((r) => (
          <Card
            key={r.key}
            size="small"
            title={<div className="text-base font-semibold truncate">{r.mode}</div>}
          >
            <div className="text-sm">
              <div className="mb-1">
                <div className="text-xs uppercase tracking-wide text-gray-500 dark:text-gray-400">
                  What it does
                </div>
                <div className="text-gray-900 dark:text-gray-100">{r.desc}</div>
              </div>
              <div>
                <div className="text-xs uppercase tracking-wide text-gray-500 dark:text-gray-400">
                  Requires
                </div>
                <div className="text-gray-900 dark:text-gray-100">{r.req}</div>
              </div>
            </div>
          </Card>
        ))}
      </div>

      <section id="roles" className="scroll-mt-24" />
      <Title level={3}>Roles & permissions</Title>

      {/* md+ : normal table */}
      <div className="hidden md:block">
        <Table
          size="small"
          columns={roleCols}
          dataSource={roleRows}
          pagination={false}
          scroll={{ x: true }}
        />
      </div>

      {/* <md : cards (no extra shadows) */}
      <div className="block md:hidden !space-y-3">
        {roleRows.map((r) => (
          <Card
            key={r.key}
            size="small"
            title={<div className="text-base font-semibold truncate">{r.role}</div>}
          >
            <div className="text-xs uppercase tracking-wide text-gray-500 dark:text-gray-400 mb-1">
              Can
            </div>
            <div className="text-sm text-gray-900 dark:text-gray-100">{r.can}</div>
          </Card>
        ))}
      </div>

      <section id="links" className="scroll-mt-24" />
      <Title level={3}>Where to next?</Title>
      <Card>
        <ul className="list-disc pl-5">
          <li>
            New here? Start with the <a href="/help/assignments/setup">Full Setup Guide</a>.
          </li>
          <li>
            Configure behaviour in <a href="/help/assignments/config">Assignment Config</a>.
          </li>
          <li>
            Upload <a href="/help/assignments/files/makefile">Makefile</a>,{' '}
            <a href="/help/assignments/files/main-files">Main</a>,{' '}
            <a href="/help/assignments/files/memo-files">Memo</a>, and optional{' '}
            <a href="/help/assignments/files/specification">Specification</a>.
          </li>
          <li>
            Learn <a href="/help/assignments/tasks">Tasks & Subsections</a> and{' '}
            <a href="/help/assignments/mark-allocator">Mark Allocation</a>.
          </li>
          <li>
            For integrity, see <a href="/help/assignments/plagiarism/moss">Plagiarism & MOSS</a>.
          </li>
          <li>
            Student view: <a href="/help/assignments/submissions/how-to-submit">How to Submit</a>.
          </li>
        </ul>
      </Card>
    </Space>
  );
}
