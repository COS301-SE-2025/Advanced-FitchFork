// src/pages/help/assignments/SetupGuide.tsx
import {
  Typography,
  Row,
  Col,
  Steps,
  Alert,
  Card,
  Tabs,
  Collapse,
  Descriptions,
  Anchor,
  Tag,
  Timeline,
  Table,
  Divider,
  Space,
  Result,
} from 'antd';
import {
  FileZipOutlined,
  SettingOutlined,
  DeploymentUnitOutlined,
  FileTextOutlined,
  SafetyOutlined,
  UploadOutlined,
  ThunderboltOutlined,
  CheckCircleOutlined,
  ExperimentOutlined,
} from '@ant-design/icons';

const { Title, Paragraph, Text, Link } = Typography;

const anchorItems = [
  { key: 'quick-start', href: '#quick-start', title: 'Quick Start' },
  { key: 'what-you-need', href: '#what-you-need', title: 'What you’ll need' },
  { key: 'concepts', href: '#concepts', title: 'Key Concepts' },
  { key: 'steps', href: '#steps', title: 'Step-by-Step' },
  { key: 'config', href: '#config', title: 'Assignment Config' },
  { key: 'files', href: '#files', title: 'Files (Main/Makefile/Memo)' },
  { key: 'memo-allocator', href: '#memo-allocator', title: 'Memo Output & Mark Allocator' },
  { key: 'submissions', href: '#submissions', title: 'Submissions & Grading' },
  { key: 'security', href: '#security', title: 'Security (PIN & IP)' },
  { key: 'coverage-complexity', href: '#coverage-complexity', title: 'Code Coverage / Complexity' },
  { key: 'ops', href: '#ops', title: 'Maintenance (Remark/Resubmit)' },
  { key: 'faq', href: '#faq', title: 'FAQ & Tips' },
];

const archiveColumns = [
  { title: 'Type', dataIndex: 'type', key: 'type', width: 180 },
  { title: 'Extension', dataIndex: 'ext', key: 'ext', width: 160 },
  { title: 'Notes', dataIndex: 'notes', key: 'notes' },
];

const archiveData = [
  { key: 'zip', type: 'ZIP', ext: '.zip', notes: 'Recommended for most cases.' },
  { key: 'tar', type: 'TAR', ext: '.tar', notes: 'Unix-friendly.' },
  { key: 'tgz', type: 'TAR+GZIP', ext: '.tgz', notes: 'Compressed tarball.' },
  { key: 'gz', type: 'GZIP', ext: '.gz', notes: 'Single-file gzip archive.' },
];

export default function SetupGuide() {
  return (
    <Row gutter={[24, 24]}>
      <Col xs={24} lg={18} className="min-w-0">
        <Space direction="vertical" size="large" className="w-full">
          <div id="quick-start" />
          <Title level={2} className="mb-0">
            Assignment Setup Guide
          </Title>
          <Paragraph className="text-gray-500 dark:text-gray-400">
            A complete, hands-on guide to preparing an assignment that runs, marks, and reports
            correctly — from files and tasks to memo output, allocator, submission policy, and
            security.
          </Paragraph>

          <Alert
            type="info"
            showIcon
            message="Best place to start"
            description={
              <span>
                If you’re new, follow the <b>Step-by-Step</b> section below in order. You can always
                jump to specifics via the TOC on the right.
              </span>
            }
          />

          <div id="what-you-need" />
          <Title level={3} className="mt-6">
            What you’ll need
          </Title>
          <Descriptions bordered size="middle" column={1}>
            <Descriptions.Item label="Role">
              <Tag color="blue">lecturer</Tag> or <Tag>assistant_lecturer</Tag> (admin also works)
            </Descriptions.Item>
            <Descriptions.Item label="Where in the UI">
              <Text code>/modules/:id/assignments/:assignment_id/config</Text>
            </Descriptions.Item>
            <Descriptions.Item label="Archives you can upload">
              <Table
                size="small"
                columns={archiveColumns}
                dataSource={archiveData}
                pagination={false}
              />
            </Descriptions.Item>
            <Descriptions.Item label="Limits & policy">
              <ul className="list-disc pl-5">
                <li>
                  Max submission size: <b>50MB</b>
                </li>
                <li>
                  Students must confirm an <b>ownership attestation</b> when submitting
                </li>
                <li>Practice submissions are optional (configurable)</li>
              </ul>
            </Descriptions.Item>
          </Descriptions>

          <div id="concepts" />
          <Title level={3}>Key Concepts</Title>
          <Row gutter={[16, 16]}>
            <Col xs={24} md={12}>
              <Card title="Tasks" bordered>
                Each assignment is split into <b>tasks</b> (Task 1, Task 2, …). Every task runs one
                command against the submitted code and produces a <b>student output</b> text file.
              </Card>
            </Col>
            <Col xs={24} md={12}>
              <Card title="Memo Output" bordered>
                For each task, you also run the official solution to generate <b>memo output</b>.
                This is the ground truth used to mark students’ outputs.
              </Card>
            </Col>
            <Col xs={24} md={12}>
              <Card title="Mark Allocator" bordered>
                The allocator (JSON) describes tasks, subsections, and point values. It’s generated
                from memo output using your <b>delimiter</b> to segment subsections.
              </Card>
            </Col>
            <Col xs={24} md={12}>
              <Card title="Execution Config" bordered>
                <Text code>config.json</Text> controls execution limits, marking options (scheme,
                attempts, pass mark, delimiter), project language, output options, GATLAM, security,
                and code coverage.
              </Card>
            </Col>
          </Row>

          <div id="steps" />
          <Title level={3}>Step-by-Step</Title>
          <Steps
            direction="vertical"
            items={[
              {
                title: 'Create the assignment',
                icon: <DeploymentUnitOutlined />,
                description:
                  'Create or open your assignment under the target module. Ensure your role has edit permissions.',
              },
              {
                title: 'Upload required files',
                icon: <FileZipOutlined />,
                description:
                  'Go to Config → Files: upload Main, Makefile, and Memo archives. (You may overwrite per-task files later.)',
              },
              {
                title: 'Define tasks & commands',
                icon: <FileTextOutlined />,
                description:
                  'Assignments → Tasks: add tasks and their shell commands (one per task). Toggle “code coverage” for coverage-only tasks.',
              },
              {
                title: 'Generate memo output',
                icon: <ThunderboltOutlined />,
                description:
                  'Run memo for all tasks using your official solution. This populates memo outputs on disk & DB.',
              },
              {
                title: 'Build the mark allocator',
                icon: <SettingOutlined />,
                description:
                  'Use your delimiter (default "&-=-&") inside memo files to mark subsection headers, then “Generate Allocator”.',
              },
              {
                title: 'Tune config & security',
                icon: <SafetyOutlined />,
                description:
                  'Config → Marking/Execution/Security: set attempts, pass mark, submission mode (Manual/GATLAM/etc.), PIN/IPs if needed.',
              },
              {
                title: 'Test with a practice submission',
                icon: <UploadOutlined />,
                description:
                  'Submit a sample archive with practice=true. Verify outputs, report, partial marks, and feedback.',
              },
              {
                title: 'Publish',
                icon: <CheckCircleOutlined />,
                description:
                  'Announce to students. Keep an eye on submissions and use Remark/Resubmit for maintenance if needed.',
              },
            ]}
          />

          <Divider />

          <div id="config" />
          <Title level={3}>Assignment Config (config.json)</Title>
          <Tabs
            items={[
              {
                key: 'marking',
                label: 'Marking Options',
                children: (
                  <Card>
                    <Paragraph>
                      Key fields from <Text code>ExecutionConfig.marking</Text>:
                    </Paragraph>
                    <ul className="list-disc pl-5">
                      <li>
                        <Text code>marking_scheme</Text>: <Tag>exact</Tag> <Tag>percentage</Tag>{' '}
                        <Tag>regex</Tag>
                      </li>
                      <li>
                        <Text code>feedback_scheme</Text>: <Tag>auto</Tag> <Tag>manual</Tag>{' '}
                        <Tag>ai</Tag>
                      </li>
                      <li>
                        <Text code>deliminator</Text>: subsection header delimiter in memo output
                        (default <Text code>&amp;-=-&amp;</Text>)
                      </li>
                      <li>
                        <Text code>grading_policy</Text>: <Tag>last</Tag> or <Tag>best</Tag>
                      </li>
                      <li>
                        <Text code>max_attempts</Text> / <Text code>limit_attempts</Text>,{' '}
                        <Text code>pass_mark</Text>, <Text code>allow_practice_submissions</Text>
                      </li>
                      <li>
                        <Text code>dissalowed_code</Text>: list of disallowed imports/snippets;
                        detected → <b>automatic 0</b> (enforced in submit route)
                      </li>
                    </ul>
                  </Card>
                ),
              },
              {
                key: 'project',
                label: 'Project & Submission Mode',
                children: (
                  <Card>
                    <Paragraph>
                      From <Text code>ExecutionConfig.project</Text> and top-level:
                    </Paragraph>
                    <ul className="list-disc pl-5">
                      <li>
                        <Text code>language</Text>: determines main filename and interpreter
                        heuristics.
                      </li>
                      <li>
                        <Text code>submission_mode</Text>: <Tag>manual</Tag> <Tag>gatlam</Tag>{' '}
                        <Tag>rng</Tag> <Tag>codecoverage</Tag>
                      </li>
                      <li>
                        <Text code>output</Text>: include <Text code>stdout</Text> (default),{' '}
                        <Text code>stderr</Text>, <Text code>retcode</Text> in captured outputs.
                      </li>
                    </ul>
                    <Alert
                      type="info"
                      showIcon
                      message="Manual vs GA (GATLAM)"
                      description={
                        <>
                          In <Text code>manual</Text> mode, the system runs your commands directly.
                          In <Text code>gatlam</Text>, the GA pipeline orchestrates code generation
                          / evolution before running — and regenerates allocator where needed.
                        </>
                      }
                    />
                  </Card>
                ),
              },
              {
                key: 'execution',
                label: 'Execution Limits',
                children: (
                  <Card>
                    <Paragraph>
                      From <Text code>ExecutionConfig.execution</Text> (resource limits):
                    </Paragraph>
                    <ul className="list-disc pl-5">
                      <li>
                        <Text code>timeout_secs</Text>, <Text code>max_memory</Text>,{' '}
                        <Text code>max_cpus</Text>, <Text code>max_uncompressed_size</Text>,{' '}
                        <Text code>max_processes</Text>
                      </li>
                    </ul>
                  </Card>
                ),
              },
              {
                key: 'security',
                label: 'Security',
                children: (
                  <Card>
                    <Paragraph>
                      From <Text code>ExecutionConfig.security</Text>:
                    </Paragraph>
                    <ul className="list-disc pl-5">
                      <li>
                        <Text code>password_enabled</Text> + <Text code>password_pin</Text> (unlock
                        gate)
                      </li>
                      <li>
                        <Text code>cookie_ttl_minutes</Text>, <Text code>bind_cookie_to_user</Text>
                      </li>
                      <li>
                        <Text code>allowed_cidrs</Text> (IP allowlist)
                      </li>
                    </ul>
                  </Card>
                ),
              },
              {
                key: 'snippet',
                label: 'Example config.json',
                children: (
                  <Card>
                    <pre className="text-xs whitespace-pre-wrap leading-5 p-3 rounded bg-gray-50 dark:bg-gray-900 overflow-auto">
                      {`{
  "execution": { "timeout_secs": 10, "max_memory": 8589934592 },
  "marking": {
    "marking_scheme": "percentage",
    "feedback_scheme": "auto",
    "deliminator": "&-=-&",
    "grading_policy": "last",
    "max_attempts": 5,
    "limit_attempts": true,
    "pass_mark": 50,
    "allow_practice_submissions": true,
    "dissalowed_code": ["import os", "System.exit"]
  },
  "project": { "language": "cpp", "submission_mode": "manual" },
  "output": { "stdout": true, "stderr": false, "retcode": true },
  "security": { "password_enabled": false, "bind_cookie_to_user": true },
  "code_coverage": { "code_coverage_required": 80 }
}`}
                    </pre>
                  </Card>
                ),
              },
            ]}
          />

          <Divider />

          <div id="files" />
          <Title level={3}>Files (Main / Makefile / Memo)</Title>
          <Row gutter={[16, 16]}>
            <Col xs={24} md={12}>
              <Card title="Main" bordered>
                Archive containing your primary source entry file(s). The language determines
                default
                <Text code>main filename</Text>. You can also generate via Interpreter (see
                GATLAM/Interpreter).
              </Card>
            </Col>
            <Col xs={24} md={12}>
              <Card title="Makefile" bordered>
                Build/run scripts or dependencies that your tasks rely on. Provided as an archive.
              </Card>
            </Col>
            <Col xs={24} md={12}>
              <Card title="Memo" bordered>
                The official solution’s code/files. The system executes tasks against this to
                produce memo outputs (ground truth).
              </Card>
            </Col>
          </Row>

          <Alert
            className="mt-3"
            type="warning"
            showIcon
            message="Upload constraints"
            description={
              <ul className="list-disc pl-5">
                <li>Only .zip, .tar, .tgz, .gz are accepted</li>
                <li>Max size: 50MB</li>
                <li>Invalid or empty uploads return 422 (Unprocessable Entity)</li>
              </ul>
            }
          />

          <Divider />

          <div id="memo-allocator" />
          <Title level={3}>Memo Output & Mark Allocator</Title>
          <Collapse
            items={[
              {
                key: 'memo',
                label: '1) Generate Memo Output',
                children: (
                  <>
                    <Paragraph>
                      Navigate to <Text code>Assignments → Memo Output</Text> and run memo for all
                      tasks. This saves <Text code>task_#_output.txt</Text> files.
                    </Paragraph>
                    <Paragraph>
                      Ensure your memo output contains headers marked by your{' '}
                      <Text code>deliminator</Text>. Example:
                    </Paragraph>
                    <pre className="text-xs whitespace-pre-wrap leading-5 p-3 rounded bg-gray-50 dark:bg-gray-900 overflow-auto">
                      {`&-=-& Output Fizz
Fizz
&-=-& Output Buzz
Buzz`}
                    </pre>
                  </>
                ),
              },
              {
                key: 'allocator',
                label: '2) Generate Mark Allocator',
                children: (
                  <>
                    <Paragraph>
                      Go to <Text code>Assignments → Mark Allocator</Text> and click{' '}
                      <Text strong>Generate</Text>. The generator counts lines per subsection (split
                      by the delimiter) and builds <Text code>allocator.json</Text> with section
                      names and point values.
                    </Paragraph>
                    <Alert
                      type="info"
                      showIcon
                      message="Coverage tasks are excluded"
                      description="Any task marked code_coverage does not contribute points in the allocator."
                    />
                  </>
                ),
              },
            ]}
          />

          <Divider />

          <div id="submissions" />
          <Title level={3}>Submissions & Grading</Title>
          <Tabs
            items={[
              {
                key: 'student-flow',
                label: 'Student flow',
                children: (
                  <Card>
                    <Timeline
                      items={[
                        {
                          color: 'blue',
                          children: (
                            <>
                              Student uploads <Text code>.zip/.tar/.tgz/.gz</Text> (≤50MB) and must
                              tick <b>ownership attestation</b>.
                            </>
                          ),
                        },
                        {
                          color: 'blue',
                          children:
                            'Submit route stores file, allocates attempt number, and triggers code execution.',
                        },
                        {
                          color: 'green',
                          children: (
                            <>
                              On success, the system marks outputs vs memo via the{' '}
                              <Text code>MarkingJob</Text>, computes earned/total, feedback, and
                              writes <Text code>submission_report.json</Text>.
                            </>
                          ),
                        },
                      ]}
                    />
                    <Alert
                      showIcon
                      type="warning"
                      message="Disallowed code = zero"
                      description={
                        <>
                          If <Text code>contains_dissalowed_code</Text> returns true, the submission
                          is graded but the final mark is forced to <b>0</b> and saved to the report
                          (with total preserved).
                        </>
                      }
                      className="mt-3"
                    />
                  </Card>
                ),
              },
              {
                key: 'modes',
                label: 'Submission modes',
                children: (
                  <Card>
                    <Paragraph>
                      <Text code>manual</Text> runs tasks as-is. <Text code>gatlam</Text>{' '}
                      orchestrates GA and regenerates allocator/memo as needed.{' '}
                      <Text code>rng</Text> is experimental. <Text code>codecoverage</Text> is for
                      coverage-only runs.
                    </Paragraph>
                  </Card>
                ),
              },
              {
                key: 'grading',
                label: 'Grading policy & feedback',
                children: (
                  <Card>
                    <ul className="list-disc pl-5">
                      <li>
                        Policy: <Text code>last</Text> or <Text code>best</Text>
                      </li>
                      <li>
                        Schemes: <Text code>exact</Text> / <Text code>percentage</Text> /{' '}
                        <Text code>regex</Text>
                      </li>
                      <li>Feedback: Auto / Manual / AI (configurable)</li>
                    </ul>
                  </Card>
                ),
              },
            ]}
          />

          <Divider />

          <div id="security" />
          <Title level={3}>Security (Unlock PIN, Cookies, IP Allowlist)</Title>
          <Card>
            <ul className="list-disc pl-5">
              <li>
                <Text code>password_enabled</Text> + <Text code>password_pin</Text> to lock the
                assignment per device/session
              </li>
              <li>
                <Text code>cookie_ttl_minutes</Text> (default 480) and{' '}
                <Text code>bind_cookie_to_user</Text> for stronger identity binding
              </li>
              <li>
                <Text code>allowed_cidrs</Text> to constrain where students can access from
              </li>
            </ul>
          </Card>

          <Divider />

          <div id="coverage-complexity" />
          <Title level={3}>Code Coverage & Complexity</Title>
          <Alert
            type="info"
            showIcon
            message="Optional extras"
            description={
              <>
                Coverage and complexity are parsed into the report when present. Coverage tasks do
                not contribute points via allocator but appear in the report for visibility.
              </>
            }
          />

          <Divider />

          <div id="ops" />
          <Title level={3}>Maintenance: Remark & Resubmit</Title>
          <Row gutter={[16, 16]}>
            <Col xs={24} md={12}>
              <Card title="Remark (regrade)">
                Re-runs marking using existing student outputs (fast). Use when only the allocator
                or comparator logic changed.
              </Card>
            </Col>
            <Col xs={24} md={12}>
              <Card title="Resubmit (reprocess)">
                Re-runs the entire pipeline (code execution → marking). Use after changing Makefile,
                commands, main/memo files, or execution config.
              </Card>
            </Col>
          </Row>

          <Divider />

          <div id="faq" />
          <Title level={3}>FAQ & Tips</Title>
          <Collapse
            items={[
              {
                key: 'q1',
                label: 'How should I structure memo output for the allocator?',
                children: (
                  <Paragraph>
                    Use your delimiter (default <Text code>&amp;-=-&amp;</Text>) before each
                    subsection header; lines below it count towards that subsection’s points.
                  </Paragraph>
                ),
              },
              {
                key: 'q2',
                label: 'Students say “Submission too large” or “No file provided”',
                children: (
                  <ul className="list-disc pl-5">
                    <li>Ensure archive ≤ 50MB and not empty</li>
                    <li>Only .zip/.tar/.tgz/.gz are accepted</li>
                    <li>
                      Verify the UI sends the file field as <Text code>file</Text>
                    </li>
                  </ul>
                ),
              },
              {
                key: 'q3',
                label: 'Partial marks look off',
                children: (
                  <ul className="list-disc pl-5">
                    <li>Confirm memo output and delimiter usage</li>
                    <li>Regenerate allocator after changing memo output</li>
                    <li>Check comparator scheme in config</li>
                  </ul>
                ),
              },
              {
                key: 'q4',
                label: 'When should I use practice submissions?',
                children: (
                  <Paragraph>
                    Great for dry-runs. They’re fully marked and reported but don’t count toward
                    attempt limits.
                  </Paragraph>
                ),
              },
            ]}
          />

          <Result
            status="success"
            title="You’re ready to build!"
            subTitle="Create tasks, generate memo output, build the allocator, set your config — then run a practice submission."
            icon={<ExperimentOutlined />}
          />
        </Space>
      </Col>

      {/* Right rail anchor */}
      <Col xs={0} lg={6}>
        <div className="sticky top-0 pt-2">
          <Anchor items={anchorItems} />
          <Card className="mt-4" size="small" title="Related">
            <ul className="list-disc pl-5">
              <li>
                <Link href="/help/assignments/files/memo-files">Memo Files</Link>
              </li>
              <li>
                <Link href="/help/assignments/mark-allocator">Mark Allocator</Link>
              </li>
              <li>
                <Link href="/help/submissions/how-to-submit">How to Submit</Link>
              </li>
              <li>
                <Link href="/help/support/troubleshooting">Troubleshooting</Link>
              </li>
            </ul>
          </Card>
        </div>
      </Col>
    </Row>
  );
}
