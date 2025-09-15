import { Typography, Card, Descriptions, Tag, Collapse, Tabs, Table, Timeline, Space } from 'antd';
import {
  FileZipOutlined,
  LoadingOutlined,
  DiffOutlined,
  PlayCircleOutlined,
} from '@ant-design/icons';
import { useEffect, useMemo } from 'react';
import { useHelpToc } from '@/context/HelpContext';
import { CodeEditor } from '@/components/common';
import { useBreadcrumbContext } from '@/context/BreadcrumbContext';

const { Title, Paragraph, Text } = Typography;

const toc = [
  { key: 'what', href: '#what', title: 'What is the Main File?' },
  { key: 'where', href: '#where', title: 'Where to Upload' },
  { key: 'how-used', href: '#how-used', title: 'How the System Uses Main' },
  { key: 'language', href: '#language', title: 'Language & Entry Filename' },
  { key: 'structure', href: '#structure', title: 'Archive Structure' },
  { key: 'args', href: '#args', title: 'Argument-Driven Tasks (CLI)' },
  { key: 'delimiter', href: '#delimiter', title: 'Subsections via Delimiter (in Main)' },
  { key: 'best', href: '#best', title: 'Best Practices' },
  { key: 'faq', href: '#faq', title: 'FAQ' },
  { key: 'trouble', href: '#trouble', title: 'Troubleshooting' },
];

const langRows = [
  {
    key: 'java',
    lang: 'Java',
    example: 'Main.java',
    notes: 'Public class Main; class name = filename.',
  },
  { key: 'cpp', lang: 'C++', example: 'Main.cpp', notes: 'Deterministic I/O; end lines with \\n.' },
];

const langCols = [
  { title: 'Language', dataIndex: 'lang', key: 'lang', width: 200 },
  { title: 'Expected entry file', dataIndex: 'example', key: 'example', width: 220 },
  { title: 'Notes', dataIndex: 'notes', key: 'notes' },
];

// Samples — delimiter is for subsections only.
const JAVA_TASKED_SAMPLE = `public class Main {
    // Delimiter prints a subsection label. Whatever text you print becomes the subsection name.
    private static void delim(String label) { System.out.println("&-=-& " + label); }

    private static void task1() {
        // Subsections inside Task 1
        delim("Step 1");
        System.out.println(42);
        delim("Step 2");
        System.out.println("OK");
    }

    private static void task2() {
        // Subsections inside Task 2
        delim("Step 1");
        System.out.println("hello");
        delim("Step 2");
        System.out.println(12);
    }

    public static void main(String[] args) {
        String task = args.length > 0 ? args[0] : "task1";
        switch (task) {
            case "task1": task1(); break;      // Tasks selected by argument + function
            case "task2": task2(); break;
            default: System.out.println(task + " is not a valid task");
        }
    }
}
`;

const CPP_TASKED_SAMPLE = `// Main.cpp
#include <iostream>
#include <string>
using namespace std;

// Delimiter prints a subsection label. Whatever text you print becomes the subsection name.
static void delim(const string& label){ cout << "&-=-& " << label << "\\n"; }

static void task1(){
  // Subsections inside Task 1
  delim("Step 1");
  cout << 42 << "\\n";
  delim("Step 2");
  cout << "OK\\n";
}

static void task2(){
  // Subsections inside Task 2
  delim("Step 1");
  cout << "hello\\n";
  delim("Step 2");
  cout << (7 + 5) << "\\n";
}

int main(int argc, char** argv){
  ios::sync_with_stdio(false); cin.tie(nullptr);
  string task = (argc > 1) ? argv[1] : "task1";
  if      (task == "task1") task1();   // Tasks selected by argument + function
  else if (task == "task2") task2();
  else cout << task << " is not a valid task\\n";
  return 0;
}
`;

export default function MainFile() {
  const { setBreadcrumbLabel } = useBreadcrumbContext();
  const ids = useMemo(() => toc.map((t) => t.href.slice(1)), []);

  useEffect(() => {
    setBreadcrumbLabel('help/assignments/files/main-files', 'Main Files');
  }, []);

  useHelpToc({
    items: toc,
    ids,
    extra: (
      <Card className="mt-4" size="small" title="Quick facts" bordered>
        <ul className="list-disc pl-5">
          <li>Archive: zip / tar / tgz / gz</li>
          <li>Max size: 50MB</li>
          <li>Used in memo & student runs</li>
          <li>
            Delimiter for subsections in Main: <Text code>&amp;-=-&amp;</Text>
          </li>
        </ul>
      </Card>
    ),
    onMountScrollToHash: true,
  });

  return (
    <Space direction="vertical" size="small" className="w-full !p-0">
      <Title level={2} className="mb-0">
        Main File
      </Title>

      <section id="what" className="scroll-mt-24" />
      <Title level={3}>What is the Main File?</Title>
      <Paragraph className="mb-0">
        <b>Main</b> is the program that runs your test cases against code. First, during{' '}
        <b>Memo generation</b>, it runs with your memo (reference) solution to create the{' '}
        <b>Memo Output</b>. Later, for <b>student runs</b>, it runs the same tests on the student’s
        submission. The two outputs are then <b>compared</b> to decide correctness and marks.
      </Paragraph>
      <Paragraph className="mt-3 mb-0">
        The <b>Main archive</b> contains exactly one entry file at the root:{' '}
        <Text code>Main.java</Text> or <Text code>Main.cpp</Text>. No other files or folders.
      </Paragraph>

      <Descriptions bordered size="middle" column={1} className="mt-3">
        <Descriptions.Item label="Accepted formats">
          <Tag>.zip</Tag> <Tag>.tar</Tag> <Tag>.tgz</Tag> <Tag>.gz</Tag> (max&nbsp;50&nbsp;MB)
        </Descriptions.Item>
        <Descriptions.Item label="Used by">
          <Tag color="blue">Memo Output</Tag> <Tag color="green">Student Submissions</Tag>
        </Descriptions.Item>
        <Descriptions.Item label="Contains">One entry file at archive root.</Descriptions.Item>
      </Descriptions>

      <section id="where" className="scroll-mt-24" />
      <Title level={3}>Where to Upload</Title>
      <Paragraph className="mb-0">
        Go to <Text code>Assignments → Config → Files</Text> and upload the <b>Main</b> archive. The{' '}
        <b>lecturer or assistant lecturer</b> uploads Main; students never upload it.
      </Paragraph>

      <section id="how-used" className="scroll-mt-24" />
      <Title level={3}>How the System Uses Main</Title>
      <Timeline
        className="mb-2"
        items={[
          { color: 'blue', dot: <FileZipOutlined />, children: 'Upload Main, Makefile, and Memo.' },
          {
            color: 'blue',
            dot: <LoadingOutlined />,
            children: 'Memo generation: run Main + Memo + Makefile → Memo Output.',
          },
          {
            color: 'green',
            dot: <PlayCircleOutlined />,
            children: 'Student run: run Main + Student + Makefile → Student Output.',
          },
          {
            color: 'gray',
            dot: <DiffOutlined />,
            children: 'Compare Student Output to Memo Output to mark.',
          },
        ]}
      />

      <section id="language" className="scroll-mt-24" />
      <Title level={3}>Language & Entry Filename</Title>
      <Card>
        <Paragraph>
          Your <Text code>config.json → project.language</Text> sets the expected entry filename.
        </Paragraph>
        <Table size="small" columns={langCols} dataSource={langRows} pagination={false} />
      </Card>

      <section id="structure" className="scroll-mt-24" />
      <Title level={3}>Archive Structure</Title>
      <Paragraph className="mb-0">
        The archive must contain <b>exactly one file at the root</b>: the entry file listed above
        (e.g., <Text code>Main.java</Text> or <Text code>Main.cpp</Text>). Do not include any
        folders or extra files.
      </Paragraph>

      <section id="args" className="scroll-mt-24" />
      <Title level={3}>Argument-Driven Tasks (CLI)</Title>
      <Card>
        <Paragraph className="mb-3">
          Let <b>Main</b> accept a single task argument (e.g., <Text code>task1</Text>,{' '}
          <Text code>task2</Text>). If no argument is provided, default to <Text code>task1</Text>.
        </Paragraph>
        <Tabs
          items={[
            {
              key: 'java',
              label: 'Java',
              children: (
                <CodeEditor
                  title="Main.java (tasks + subsection delimiter)"
                  language="java"
                  value={JAVA_TASKED_SAMPLE}
                  height={520}
                  readOnly
                  minimal
                  fitContent
                  showLineNumbers={false}
                  hideCopyButton
                />
              ),
            },
            {
              key: 'cpp',
              label: 'C++',
              children: (
                <CodeEditor
                  title="Main.cpp (tasks + subsection delimiter)"
                  language="cpp"
                  value={CPP_TASKED_SAMPLE}
                  height={420}
                  readOnly
                  minimal
                  fitContent
                  showLineNumbers={false}
                  hideCopyButton
                />
              ),
            },
          ]}
        />
        <Paragraph className="mt-3 mb-0">
          In your <a href="/help/assignments/files/makefile">Makefile</a>, define a target for each
          task so you can invoke <Text code>task1</Text>, <Text code>task2</Text>, etc., directly.
        </Paragraph>
      </Card>

      <section id="delimiter" className="scroll-mt-24" />
      <Title level={3}>Subsections via Delimiter (in Main)</Title>
      <Card>
        <Paragraph className="mb-2">
          The delimiter <Text code>&amp;-=-&amp;</Text> is used <b>only inside Main</b> to mark{' '}
          <b>subtasks/subsections</b> within the current task. <u>Do not</u> use it to identify the
          task itself — tasks are chosen by the CLI argument and your functions (e.g.,{' '}
          <Text code>task1</Text>, <Text code>task2</Text>). Whatever you print after the delimiter
          becomes the subsection name in marking.
        </Paragraph>
        <CodeEditor
          language="plaintext"
          value={`&-=-& Step 1
42
&-=-& Step 2
OK
&-=-& Step 1
hello
&-=-& Step 2
12`}
          height={200}
          readOnly
          minimal
          title="Example stdout (subsections only)"
          fitContent
          showLineNumbers={false}
          hideCopyButton
        />
      </Card>

      <section id="best" className="scroll-mt-24" />
      <Title level={3}>Best Practices</Title>
      <ul className="list-disc pl-5">
        <li>Single entry file at archive root (no folders).</li>
        <li>Avoid non-determinism in output (timestamps, randomness, machine paths).</li>
        <li>Keep output concise — it becomes the basis for comparison.</li>
        <li>Version-control the source that you zip as Main.</li>
      </ul>

      <section id="faq" className="scroll-mt-24" />
      <Title level={3}>FAQ</Title>
      <Collapse
        items={[
          {
            key: 'f1',
            label: 'Does Main “use” Memo when generating Memo Output?',
            children: (
              <Paragraph>
                Yes. During memo generation, the platform runs <b>Main + Memo + Makefile</b>. Your
                Main orchestrates execution and the Memo provides the reference implementation.
              </Paragraph>
            ),
          },
          {
            key: 'f2',
            label: 'Do students see the Main archive?',
            children: <Paragraph>No. Students never upload or see your Main directly.</Paragraph>,
          },
          {
            key: 'f3',
            label: 'How do disallowed imports affect Main?',
            children: (
              <Paragraph>
                Disallowed code is enforced on <b>student submissions</b>. Make expectations clear
                so students avoid banned libraries.
              </Paragraph>
            ),
          },
        ]}
      />

      <section id="trouble" className="scroll-mt-24" />
      <Title level={3}>Troubleshooting</Title>
      <Collapse
        items={[
          {
            key: 't1',
            label: '“No .zip/.tar/.tgz/.gz file found”',
            children: (
              <Paragraph>
                Upload a supported archive under <Text code>Assignments → Config → Files</Text>.
                Keep it under <b>50MB</b>.
              </Paragraph>
            ),
          },
          {
            key: 't2',
            label: 'Build/run fails during memo generation or student runs',
            children: (
              <ul className="list-disc pl-5">
                <li>
                  Check task commands under <Text code>Assignments → Tasks</Text>.
                </li>
                <li>
                  Verify execution limits in <Text code>config.execution</Text> (timeout, CPUs,
                  memory).
                </li>
              </ul>
            ),
          },
          {
            key: 't3',
            label: 'Outputs vary across runs',
            children: (
              <Paragraph>
                Remove non-determinism (timestamps, RNG) from Main or redirect those logs away from
                stdout.
              </Paragraph>
            ),
          },
        ]}
      />
    </Space>
  );
}
