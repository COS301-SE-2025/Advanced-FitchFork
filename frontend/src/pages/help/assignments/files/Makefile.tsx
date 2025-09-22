// src/pages/help/assignments/files/Makefile.tsx
import {
  Typography,
  Card,
  Descriptions,
  Tag,
  Collapse,
  Steps,
  Tabs,
  Table,
  Timeline,
  Space,
  Row,
  Col,
  Statistic,
} from 'antd';
import {
  FileZipOutlined,
  BuildOutlined,
  PlayCircleOutlined,
  SettingOutlined,
} from '@ant-design/icons';
import { useEffect, useMemo } from 'react';
import { useHelpToc } from '@/context/HelpContext';
import { CodeEditor } from '@/components/common';
import { useBreadcrumbContext } from '@/context/BreadcrumbContext';

const { Title, Paragraph, Text } = Typography;

const toc = [
  { key: 'what', href: '#what', title: 'What is the Makefile archive?' },
  { key: 'why', href: '#why', title: 'Why separate from Main & Memo' },
  { key: 'where', href: '#where', title: 'Where to Upload' },
  { key: 'how', href: '#how', title: 'How the Runner Uses It' },
  { key: 'examples', href: '#examples', title: 'Examples (C++ / Java)' },
  { key: 'tasks', href: '#tasks', title: 'Defining Task Commands' },
  { key: 'output', href: '#output', title: 'What Output is Captured' },
  { key: 'limits', href: '#limits', title: 'Execution Limits' },
  { key: 'best', href: '#best', title: 'Best Practices' },
  { key: 'trouble', href: '#trouble', title: 'Troubleshooting' },
];

const langRows = [
  {
    key: 'cpp',
    lang: 'C++',
    target: 'make task1',
    note: 'Build with g++ and run ./bin/app task1 (or task2, …).',
  },
  {
    key: 'java',
    lang: 'Java',
    target: 'make task1',
    note: 'Compile to ./out and run: java -cp out Main task1 (or task2, …).',
  },
];

const langCols = [
  { title: 'Language', dataIndex: 'lang', key: 'lang', width: 160 },
  { title: 'Typical Task Command', dataIndex: 'target', key: 'target', width: 240 },
  { title: 'Notes', dataIndex: 'note', key: 'note' },
];

/** Makefile-only examples: targets = Tasks; each target calls Main with the task arg. */
const MAKEFILE_CPP = `# Makefile (C++)
CXX := g++
CXXFLAGS := -O2 -std=c++17 -Wall -Wextra
SRC := $(wildcard src/*.cpp)
BIN := bin/app

.PHONY: build task1 task2 clean
build: $(BIN)

$(BIN): $(SRC)
\t@mkdir -p bin
\t$(CXX) $(CXXFLAGS) -o $@ $(SRC)

# Each Task target runs Main with the task argument
task1: build
\t./$(BIN) task1

task2: build
\t./$(BIN) task2

clean:
\trm -rf bin
`;

const MAKEFILE_JAVA = `# Makefile (Java)
JAVAC := javac
JAVA := java
SRC := $(shell find src -name "*.java")
OUT := out
MAIN := Main

.PHONY: build task1 task2 clean
build:
\t@mkdir -p $(OUT)
\t$(JAVAC) -d $(OUT) $(SRC)

# Each Task target runs Main with the task argument
task1: build
\t$(JAVA) -cp $(OUT) $(MAIN) task1

task2: build
\t$(JAVA) -cp $(OUT) $(MAIN) task2

clean:
\trm -rf $(OUT)
`;

export default function MakefileHelp() {
  const { setBreadcrumbLabel } = useBreadcrumbContext();
  const ids = useMemo(() => toc.map((t) => t.href.slice(1)), []);

  useEffect(() => {
    setBreadcrumbLabel('help/assignments/files/makefile', 'Makefile');
  }, []);

  useHelpToc({
    items: toc,
    ids,
    extra: (
      <Card className="mt-4" size="small" title="Quick facts" bordered>
        <ul className="list-disc pl-5">
          <li>
            Archive contains <b>only</b> a root-level <Text code>Makefile</Text>
          </li>
          <li>
            Make <b>targets = Tasks</b> (e.g., <Text code>make task1</Text>)
          </li>
          <li>
            Targets build then run <Text code>Main &lt;task&gt;</Text>
          </li>
          <li>Regenerate Memo Output after Makefile changes</li>
        </ul>
      </Card>
    ),
    onMountScrollToHash: true,
  });

  return (
    <Space direction="vertical" size="small" className="w-full !p-0">
      <Title level={2} className="mb-0">
        Makefile
      </Title>

      <section id="what" className="scroll-mt-24" />
      <Title level={3}>What is the Makefile archive?</Title>
      <Paragraph className="mb-0">
        The <b>Makefile archive</b> is a single archive that contains <b>only</b> a{' '}
        <Text code>Makefile</Text> at its root. The Makefile defines <b>targets</b> that correspond
        to your assignment <b>Tasks</b> and invoke your <b>Main</b> program with the proper task
        argument (e.g., <Text code>task1</Text>, <Text code>task2</Text>).
      </Paragraph>
      <Descriptions bordered size="middle" column={1} className="mt-3">
        <Descriptions.Item label="Accepted formats">
          <Tag>.zip</Tag> <Tag>.tar</Tag> <Tag>.tgz</Tag> <Tag>.gz</Tag>
        </Descriptions.Item>
        <Descriptions.Item label="Must contain">
          Exactly one file at the archive root: <Text code>Makefile</Text> (no folders or extras).
        </Descriptions.Item>
      </Descriptions>

      <section id="why" className="scroll-mt-24" />
      <Title level={3}>Why separate from Main & Memo</Title>
      <Timeline
        className="mb-2"
        items={[
          {
            color: 'blue',
            dot: <FileZipOutlined />,
            children: (
              <>
                You upload three archives: <b>Main</b> (runtime & CLI), <b>Makefile</b> (build/run
                targets), and <b>Memo</b> (reference solution).
              </>
            ),
          },
          {
            color: 'blue',
            dot: <BuildOutlined />,
            children: (
              <>The Makefile keeps “how to build & run” stable across memo and student runs.</>
            ),
          },
          {
            color: 'green',
            dot: <PlayCircleOutlined />,
            children: <>Each Task executes a Make target that runs Main for that specific task.</>,
          },
        ]}
      />

      <section id="where" className="scroll-mt-24" />
      <Title level={3}>Where to Upload</Title>
      <Paragraph className="mb-0">
        Go to <Text code>Assignments → Config → Files</Text> and upload the archive into the{' '}
        <b>Makefile</b> slot. The archive must have a single root-level <Text code>Makefile</Text>.
      </Paragraph>

      <section id="how" className="scroll-mt-24" />
      <Title level={3}>How the Runner Uses It</Title>
      <Paragraph className="mb-2">
        For each Task you configure, the platform runs the corresponding <b>Make target</b>. Those
        targets should <b>build</b> (if needed) and then run <Text code>Main &lt;task&gt;</Text> so
        your CLI in Main selects the correct task.
      </Paragraph>
      <Descriptions bordered size="middle" column={1}>
        <Descriptions.Item label="Where it runs">
          Main, Makefile, and Memo/Student code are extracted together into an isolated workspace.
        </Descriptions.Item>
        <Descriptions.Item label="Task command">
          Set each Task’s command to a Make target (e.g., <Text code>make task1</Text>).
        </Descriptions.Item>
      </Descriptions>

      <section id="examples" className="scroll-mt-24" />
      <Title level={3}>Examples</Title>
      <Card>
        <Tabs
          items={[
            {
              key: 'cpp',
              label: 'C++',
              children: (
                <CodeEditor
                  title="Makefile (C++)"
                  language="makefile"
                  value={MAKEFILE_CPP}
                  height={360}
                  readOnly
                  minimal
                  fitContent
                  showLineNumbers={false}
                  hideCopyButton
                />
              ),
            },
            {
              key: 'java',
              label: 'Java',
              children: (
                <CodeEditor
                  title="Makefile (Java)"
                  language="makefile"
                  value={MAKEFILE_JAVA}
                  height={320}
                  readOnly
                  minimal
                  fitContent
                  showLineNumbers={false}
                  hideCopyButton
                />
              ),
            },
            {
              key: 'table',
              label: 'Quick reference',
              children: (
                <Table columns={langCols} dataSource={langRows} pagination={false} size="small" />
              ),
            },
          ]}
        />
      </Card>

      <section id="tasks" className="scroll-mt-24" />
      <Title level={3}>Defining Task Commands</Title>
      <Card>
        <Paragraph>
          Create one Make target per Task (e.g., <Text code>task1</Text>, <Text code>task2</Text>).
          Each target should build if needed and then run <Text code>Main &lt;task&gt;</Text>.
        </Paragraph>
        <Steps
          direction="vertical"
          items={[
            {
              title: 'Open Tasks',
              description: (
                <>
                  Go to <Text code>Assignments → Tasks</Text>.
                </>
              ),
            },
            {
              title: 'Set command',
              description: (
                <>
                  Use a Make target, e.g., <Text code>make task1</Text> or{' '}
                  <Text code>make task2</Text>.
                </>
              ),
            },
            {
              title: 'Save & test',
              description: <>Generate Memo Output to validate the full build–run cycle.</>,
            },
          ]}
        />
      </Card>

      <section id="output" className="scroll-mt-24" />
      <Title level={3}>What Output is Captured</Title>
      <Card>
        <Paragraph className="mb-0">
          Output capture (Standard output, Error output, Exit status) is configured in{' '}
          <a href="/help/assignments/config">Assignment Config</a> → Output. Standard output is
          enabled by default; toggle the others there as needed.
        </Paragraph>
      </Card>

      <section id="limits" className="scroll-mt-24" />
      <Title level={3}>Execution Limits</Title>
      <Card>
        <Row gutter={16}>
          <Col xs={12} sm={8}>
            <Statistic title="Time limit (s)" value={10} prefix={<SettingOutlined />} />
          </Col>
          <Col xs={12} sm={8}>
            <Statistic title="Memory limit" value="8 GiB" prefix={<SettingOutlined />} />
          </Col>
          <Col xs={12} sm={8}>
            <Statistic title="CPUs" value={2} prefix={<SettingOutlined />} />
          </Col>
        </Row>
        <Paragraph className="mt-3 mb-0">
          These come from <a href="/help/assignments/config">Assignment Config</a> → Execution.
          Increase only if necessary.
        </Paragraph>
      </Card>

      <section id="best" className="scroll-mt-24" />
      <Title level={3}>Best Practices</Title>
      <ul className="list-disc pl-5">
        <li>
          <b>One file only</b>: the archive contains just a root-level <Text code>Makefile</Text>.
        </li>
        <li>
          <b>Targets mirror Tasks</b>: <Text code>task1</Text>, <Text code>task2</Text>, …
        </li>
        <li>
          <b>Deterministic output</b>: avoid timestamps/randomness in what you print.
        </li>
        <li>
          <b>Quiet stdout</b>: only print what’s needed for marking.
        </li>
        <li>
          <b>Separate build & run</b>: compile in <Text code>build</Text>, run in per-task targets.
        </li>
      </ul>

      <section id="trouble" className="scroll-mt-24" />
      <Title level={3}>Troubleshooting</Title>
      <Collapse
        items={[
          {
            key: 't1',
            label: '“No .zip/.tar/.tgz/.gz file found in makefile slot”',
            children: (
              <Paragraph>
                Upload exactly one archive to the Makefile slot and ensure it contains a single
                root-level <Text code>Makefile</Text>.
              </Paragraph>
            ),
          },
          {
            key: 't2',
            label: 'Make target not found',
            children: (
              <ul className="list-disc pl-5">
                <li>
                  Define targets that match your Task commands (e.g., <Text code>task1</Text>).
                </li>
                <li>Use relative paths that match the extracted layout.</li>
              </ul>
            ),
          },
          {
            key: 't3',
            label: 'Timeouts or OOM during build/run',
            children: (
              <ul className="list-disc pl-5">
                <li>Optimize compile flags; keep dependencies and artifacts small.</li>
                <li>
                  Adjust the time/memory/CPU limits in{' '}
                  <a href="/help/assignments/config">Assignment Config</a> → Execution if truly
                  needed.
                </li>
              </ul>
            ),
          },
          {
            key: 't4',
            label: 'Outputs differ across runs',
            children: (
              <Paragraph>
                Remove nondeterminism (RNG, timestamps, machine paths). Stable output is required
                for comparison with the Memo Output.
              </Paragraph>
            ),
          },
        ]}
      />
    </Space>
  );
}
