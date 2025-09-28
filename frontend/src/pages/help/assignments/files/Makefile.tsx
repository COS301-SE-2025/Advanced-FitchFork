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
import { useViewSlot } from '@/context/ViewSlotContext';

const { Title, Paragraph, Text } = Typography;

const toc = [
  { key: 'overview', href: '#overview', title: 'Why the Makefile Matters' },
  { key: 'lifecycle', href: '#lifecycle', title: 'How Fitchfork Uses the Makefile' },
  { key: 'requirements', href: '#requirements', title: 'Archive & Target Requirements' },
  { key: 'storage', href: '#storage', title: 'After You Upload' },
  { key: 'tasks', href: '#tasks', title: 'Mapping Tasks to Targets' },
  { key: 'examples', href: '#examples', title: 'Examples & Patterns' },
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
  {
    key: 'python',
    lang: 'Python',
    target: 'make task1',
    note: 'Install deps if needed, then call python3 main.py task1.',
  },
  {
    key: 'rust',
    lang: 'Rust',
    target: 'make task1',
    note: 'Use cargo build --release; run ./target/release/app task1.',
  },
  {
    key: 'mixed',
    lang: 'Multi-language',
    target: 'make task1',
    note: 'Orchestrate scripts or binaries; ensure relative paths match the extracted workspace.',
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
  const { setValue, setBackTo } = useViewSlot();

  useEffect(() => {
    setValue(
      <Typography.Text className="text-base font-medium text-gray-900 dark:text-gray-100 truncate">
        Makefile
      </Typography.Text>,
    );
    setBackTo('/help');
  }, []);

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
            Upload under <Text code>Assignments → Config → Files</Text>
          </li>
          <li>
            Archive format: .zip / .tar / .tgz / .gz (≤50&nbsp;MB) with a single{' '}
            <Text code>Makefile</Text>
          </li>
          <li>
            Make targets mirror Tasks; each runs <Text code>Main &lt;task&gt;</Text>
          </li>
          <li>Regenerate Memo Output whenever the Makefile changes</li>
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

      <section id="overview" className="scroll-mt-24" />
      <Title level={3}>Why the Makefile Matters</Title>
      <Paragraph className="mb-0">
        The <strong>Makefile</strong> teaches Fitchfork how to build and execute your tests. Main
        drives the logic, Memo supplies the reference answers, and the Makefile stitches everything
        together by turning each Task into a repeatable shell command. Keeping it separate lets you
        tweak build flags or tooling without touching Main or Memo archives.
      </Paragraph>
      <Paragraph className="mt-3 mb-0">
        Upload the Makefile archive under <Text code>Assignments → Config → Files</Text>. Lecturers
        or assistant lecturers own this upload. In Manual submission mode it is required; in GATLAM
        mode the Interpreter is responsible for compiling/running, so the Makefile can be left
        empty.
      </Paragraph>

      <Descriptions bordered size="middle" column={1} className="mt-3">
        <Descriptions.Item label="Archive contents">
          Exactly one root-level <Text code>Makefile</Text> (no folders, binaries, or helper files).
        </Descriptions.Item>
        <Descriptions.Item label="Formats">
          <Tag>.zip</Tag> <Tag>.tar</Tag> <Tag>.tgz</Tag> <Tag>.gz</Tag> (≤50&nbsp;MB)
        </Descriptions.Item>
      </Descriptions>

      <section id="lifecycle" className="scroll-mt-24" />
      <Title level={3}>How Fitchfork Uses the Makefile</Title>
      <Timeline
        className="mb-2"
        items={[
          {
            color: 'blue',
            dot: <FileZipOutlined />,
            children:
              'Upload Main, Makefile, and Memo archives. Each is validated to ensure a .zip/.tar/.tgz/.gz exists.',
          },
          {
            color: 'blue',
            dot: <BuildOutlined />,
            children:
              'Generate Memo Output: Fitchfork unpacks the trio into an isolated workspace and runs your Make targets to capture reference output.',
          },
          {
            color: 'green',
            dot: <PlayCircleOutlined />,
            children:
              'Student runs call the same Make targets in fresh containers, ensuring students build with the exact flags you expect.',
          },
          {
            color: 'gray',
            dot: <SettingOutlined />,
            children:
              'If the Makefile archive is missing or lacks a .zip, memo generation and student attempts fail fast with an explicit “Makefile archive not found” error.',
          },
        ]}
      />

      <section id="requirements" className="scroll-mt-24" />
      <Title level={3}>Archive &amp; Target Requirements</Title>
      <Paragraph className="mb-0">
        Treat the Makefile as the authoritative source for build commands. Set up phony targets that
        compile in a <Text code>build</Text> step (if necessary) and then run{' '}
        <Text code>Main &lt;task&gt;</Text>. Use relative paths because Fitchfork extracts
        everything into the workspace root before executing targets.
      </Paragraph>

      <section id="storage" className="scroll-mt-24" />
      <Title level={3}>After You Upload</Title>
      <Paragraph className="mb-2">
        Fitchfork stores the archive in the assignment’s <Text code>makefile/</Text> folder and
        tracks it in the assignment files list. Readiness checks, the Setup Checklist, and memo
        generation all look for that stored archive. Uploading a new version overwrites the existing
        file; the next memo generation or student run uses it immediately, so keep prior versions in
        source control if you may need to revert.
      </Paragraph>
      <Descriptions bordered size="middle" column={1}>
        <Descriptions.Item label="Readiness flag">
          Assignments register <Text code>makefile_present</Text> once a valid archive is stored.
          Manual mode requires this flag to be true before the assignment is considered ready.
        </Descriptions.Item>
        <Descriptions.Item label="Validation">
          Missing or empty Makefile directories trigger “Makefile archive (.zip) not found” errors
          during memo generation and student attempts.
        </Descriptions.Item>
      </Descriptions>

      <section id="tasks" className="scroll-mt-24" />
      <Title level={3}>Mapping Tasks to Targets</Title>
      <Card>
        <Paragraph>
          Create one Make target per Task (e.g., <Text code>task1</Text>, <Text code>task2</Text>).
          Each target should prepare any build artefacts, then invoke{' '}
          <Text code>Main &lt;task&gt;</Text>. The Tasks screen simply shells out to the command you
          provide, so keeping the naming aligned avoids “target not found” failures at run time.
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

      <section id="examples" className="scroll-mt-24" />
      <Title level={3}>Examples &amp; Patterns</Title>
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
                <>
                  {/* md+ : normal table */}
                  <div className="hidden md:block">
                    <Table
                      columns={langCols}
                      dataSource={langRows}
                      pagination={false}
                      size="small"
                      scroll={{ x: true }}
                    />
                  </div>

                  {/* <md : cards (no extra shadows) */}
                  <div className="block md:hidden mt-2 !space-y-3">
                    {langRows.map((r) => (
                      <Card
                        key={r.key}
                        size="small"
                        title={<div className="text-base font-semibold truncate">{r.lang}</div>}
                      >
                        <div className="text-xs uppercase tracking-wide text-gray-500 dark:text-gray-400 mb-1">
                          Typical Task Command
                        </div>
                        <div className="text-sm text-gray-900 dark:text-gray-100 mb-2">
                          {r.target}
                        </div>

                        <div className="text-xs uppercase tracking-wide text-gray-500 dark:text-gray-400 mb-1">
                          Notes
                        </div>
                        <div className="text-sm text-gray-900 dark:text-gray-100">{r.note}</div>
                      </Card>
                    ))}
                  </div>
                </>
              ),
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
        <li>
          <b>Stay self-contained</b>: rely on files from Main/Memo archives; avoid network installs
          or absolute paths.
        </li>
        <li>
          <b>Use phony targets</b> so repeated runs do not skip commands because artefacts already
          exist.
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
                root-level <Text code>Makefile</Text>. The readiness checklist will flip
                <Text code>makefile_present</Text> back to true once the new archive is stored.
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
