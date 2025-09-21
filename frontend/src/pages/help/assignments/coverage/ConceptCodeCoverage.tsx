// src/pages/help/assignments/coverage/ConceptCodeCoverage.tsx
import { useEffect, useMemo } from 'react';
import { Typography, Card, Alert, Descriptions, Steps, Tag } from 'antd';
import { useHelpToc } from '@/context/HelpContext';
import { useBreadcrumbContext } from '@/context/BreadcrumbContext';

const { Title, Paragraph, Text } = Typography;

const toc = [
  { key: 'overview', href: '#overview', title: 'What it is' },
  { key: 'workflow', href: '#workflow', title: 'How to set it up' },
  { key: 'report', href: '#report', title: 'Report format' },
  { key: 'integration', href: '#integration', title: 'How it integrates' },
  { key: 'lang', href: '#lang', title: 'Language tooling' },
  { key: 'trouble', href: '#trouble', title: 'Troubleshooting' },
];

export default function ConceptCodeCoverage() {
  const { setBreadcrumbLabel } = useBreadcrumbContext();
  const ids = useMemo(() => toc.map((t) => t.href.slice(1)), []);

  useEffect(() => {
    setBreadcrumbLabel('help/assignments/code-coverage', 'Code Coverage');
  }, []);

  useHelpToc({ items: toc, ids, onMountScrollToHash: true });

  return (
    <div className="space-y-4">
      <Title level={2} className="mb-0">
        Code Coverage
      </Title>

      <section id="overview" className="scroll-mt-24" />
      <Title level={3}>What it is</Title>
      <Paragraph className="mb-0">
        Code coverage tasks let you run instrumented builds and tests to report how much of your
        code was executed. Coverage is informative and <b>does not contribute to marks</b> or the
        <b> Mark Allocator</b>. It appears alongside submission results when available.
      </Paragraph>
      <Alert
        className="mt-2"
        type="info"
        showIcon
        message="Coverage tasks are special"
        description={
          <>
            When a task is marked <Text code>code_coverage: true</Text>:
            <ul className="list-disc pl-5 mt-1">
              <li>
                It is <b>excluded</b> from memo-output generation
              </li>
              <li>It does not produce subsections or points in the allocator</li>
              <li>Its outputs are treated as coverage artifacts for reporting</li>
            </ul>
          </>
        }
      />

      <section id="workflow" className="scroll-mt-24" />
      <Title level={3}>How to set it up</Title>
      <Steps
        direction="vertical"
        items={[
          {
            title: 'Create or edit a task',
            description: 'Assignments → Tasks → Add/Edit. Give it a clear name (e.g., “Coverage”).',
          },
          {
            title: 'Enable Code Coverage for the task',
            description: 'Toggle the task field code_coverage = true.',
          },
          {
            title: 'Provide a coverage command',
            description:
              'Set the task command to build with instrumentation and run the tests (e.g., lcov/gcov for C++).',
          },
          {
            title: 'Run a practice submission',
            description:
              'Submit once to verify the pipeline runs and produces coverage artifacts as expected.',
          },
        ]}
      />

      <section id="report" className="scroll-mt-24" />
      <Title level={3}>Report format</Title>
      <Paragraph>
        The platform expects a normalized JSON when integrating coverage into the submission report
        (summary plus per-file details). Internally, coverage tools can emit their native formats;
        the system may transform them to this schema:
      </Paragraph>
      <Card size="small" className="bg-gray-50 dark:bg-gray-900">
        <pre className="text-xs whitespace-pre-wrap leading-5 m-0">
          {`{
  "generated_at": "2025-01-01T12:00:00Z",
  "summary": { "total_files": 12, "total_lines": 1042, "covered_lines": 873, "coverage_percent": 83.8 },
  "files": [
    { "path": "src/lib.rs", "total_lines": 200, "covered_lines": 180, "coverage_percent": 90.0 },
    { "path": "src/main.rs", "total_lines": 60, "covered_lines": 42, "coverage_percent": 70.0 }
  ]
}`}
        </pre>
      </Card>
      <Alert
        className="!mt-2"
        type="warning"
        showIcon
        message="Marks vs coverage"
        description="Coverage is informational; it does not affect earned/total marks or allocator points."
      />

      <section id="integration" className="scroll-mt-24" />
      <Title level={3}>How it integrates</Title>
      <Descriptions bordered size="middle" column={1} className="!mt-2">
        <Descriptions.Item label="Tasks">
          Mark a task with <Text code>code_coverage: true</Text>. The memo generator skips these
          tasks, and they are excluded from mark allocation.
        </Descriptions.Item>
        <Descriptions.Item label="Config">
          <Text code>config.json → code_coverage.code_coverage_required</Text> lets you record a
          target percentage (default <Text code>80</Text>). This is a guideline and may surface as a
          warning; it does not gate marking.
        </Descriptions.Item>
        <Descriptions.Item label="Submission report">
          If present, coverage appears under <Text code>data.code_coverage</Text> in the submission
          JSON. It is optional and shown alongside tasks.
        </Descriptions.Item>
        <Descriptions.Item label="Allocator">
          Coverage tasks do <b>not</b> contribute points. They are visible in reports only.
        </Descriptions.Item>
      </Descriptions>

      <section id="lang" className="scroll-mt-24" />
      <Title level={3}>Language tooling</Title>
      <Card>
        <ul className="list-disc pl-5">
          <li>
            <b>C/C++</b>: compile with <Text code>-fprofile-arcs -ftest-coverage -O0</Text>; run
            tests; generate text or lcov output. The runtime container includes{' '}
            <Text code>lcov</Text>.
          </li>
          <li>
            <b>Rust</b>: consider <Text code>cargo llvm-cov</Text> or per-task coverage command.
          </li>
          <li>
            <b>Java</b>: tooling like JaCoCo can be invoked in the task command. Normalization to
            the JSON schema may be required.
          </li>
        </ul>
      </Card>
      <Alert
        className="!mt-2"
        type="info"
        showIcon
        message="Tip"
        description={
          <>
            Keep coverage commands isolated in a dedicated task (e.g., <Tag>Coverage</Tag>). This
            keeps normal marking tasks simple and reproducible.
          </>
        }
      />

      <section id="trouble" className="scroll-mt-24" />
      <Title level={3}>Troubleshooting</Title>
      <Card size="small">
        <ul className="list-disc pl-5">
          <li>
            Coverage is missing in the report: confirm the coverage task ran and produced artifacts;
            ensure the command exits successfully.
          </li>
          <li>
            Memo generation skips my coverage task: expected. Coverage tasks don’t have memo output.
          </li>
          <li>
            Coverage percent looks wrong: check your tool’s include/exclude patterns and that test
            binaries ran with instrumentation enabled.
          </li>
        </ul>
      </Card>
    </div>
  );
}
