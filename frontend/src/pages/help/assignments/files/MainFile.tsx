import { Typography, Card, Descriptions, Tag, Collapse, Tabs, Table, Timeline, Space } from 'antd';
import {
  FileZipOutlined,
  LoadingOutlined,
  DiffOutlined,
  PlayCircleOutlined,
} from '@ant-design/icons';
import { useEffect, useMemo } from 'react';
import { useHelpToc } from '@/context/HelpContext';
import { CodeEditor, GatlamLink } from '@/components/common';
import { useBreadcrumbContext } from '@/context/BreadcrumbContext';
import { useViewSlot } from '@/context/ViewSlotContext';

const { Title, Paragraph, Text } = Typography;

const toc = [
  { key: 'overview', href: '#overview', title: 'Why Main Matters' },
  { key: 'lifecycle', href: '#lifecycle', title: 'How Fitchfork Runs Main' },
  { key: 'requirements', href: '#requirements', title: 'Archive & Entry Requirements' },
  { key: 'storage', href: '#storage', title: 'After You Upload' },
  { key: 'tasks', href: '#tasks', title: 'Task Arguments & CLI' },
  { key: 'delimiter', href: '#delimiter', title: 'Subsections with the Delimiter' },
  { key: 'best', href: '#best', title: 'Best Practices' },
  { key: 'faq', href: '#faq', title: 'FAQ' },
  { key: 'trouble', href: '#trouble', title: 'Troubleshooting' },
];

const langRows = [
  {
    key: 'java',
    lang: 'Java',
    example: 'Main.java',
    notes: 'Public class Main; keep main(String[] args) entry point.',
  },
  {
    key: 'cpp',
    lang: 'C++',
    example: 'Main.cpp',
    notes: 'Flush stdout; disable sync_with_stdio for performance.',
  },
  {
    key: 'c',
    lang: 'C',
    example: 'main.c',
    notes: 'Provide an int main(void) entry point and flush output.',
  },
  {
    key: 'python',
    lang: 'Python',
    example: 'main.py',
    notes: 'Read task argument from sys.argv; avoid random stdout ordering.',
  },
  {
    key: 'rust',
    lang: 'Rust',
    example: 'main.rs',
    notes: 'Use std::env::args() for task selection; cargo builds via Makefile.',
  },
];

const langCols = [
  { title: 'Language', dataIndex: 'lang', key: 'lang', width: 200 },
  { title: 'Expected entry file', dataIndex: 'example', key: 'example', width: 220 },
  { title: 'Notes', dataIndex: 'notes', key: 'notes' },
];

// Samples — delimiter is for subsections only.
const JAVA_TASKED_SAMPLE = `public class Main {
    // Delimiter prints a subsection label. Whatever text you print becomes the subsection name.
    private static void delim(String label) { System.out.println("### " + label); }

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
static void delim(const string& label){ cout << "### " << label << "\\n"; }

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
  const { setValue, setBackTo } = useViewSlot();

  useEffect(() => {
    setValue(
      <Typography.Text className="text-base font-medium text-gray-900 dark:text-gray-100 truncate">
        Main File
      </Typography.Text>,
    );
    setBackTo('/help');
  }, []);

  useEffect(() => {
    setBreadcrumbLabel('help/assignments/files/main-files', 'Main Files');
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
          <li>Archive format: .zip / .tar / .tgz / .gz (≤50&nbsp;MB)</li>
          <li>Run order: Memo generation first, then every student run</li>
          <li>
            Subsection delimiter inside Main: <Text code>&amp;-=-&amp;</Text>
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

      <section id="overview" className="scroll-mt-24" />
      <Title level={3}>Why Main Matters</Title>
      <Paragraph className="mb-0">
        <strong>Main</strong> is the harness Fitchfork uses to exercise every submission. It couples
        with your <strong>Makefile</strong> and <strong>Memo</strong> archives to generate reference
        outputs, and the same trio is replayed for each student attempt. Because Main controls the
        test flow, it also decides how tasks are invoked, which files are compiled, and how marks
        are segmented.
      </Paragraph>
      <Paragraph className="mt-3 mb-0">
        Upload Main under <Text code>Assignments → Config → Files</Text>. Lecturers (or assistant
        lecturers) own this archive; students never see or submit it. In Manual submission mode Main
        is required. If you switch the assignment to{' '}
        <GatlamLink tone="inherit" icon={false} underline={false}>
          GATLAM
        </GatlamLink>
        , the <strong>Interpreter</strong>{' '}
        replaces Main and this archive can stay empty.
      </Paragraph>

      <Descriptions bordered size="middle" column={1} className="mt-3">
        <Descriptions.Item label="Runs with">
          <Tag color="blue">Memo Output</Tag> <Tag color="green">Student Submissions</Tag>
        </Descriptions.Item>
        <Descriptions.Item label="Upload role">
          Lecturer / Assistant Lecturer only
        </Descriptions.Item>
        <Descriptions.Item label="Archive formats">
          <Tag>.zip</Tag> <Tag>.tar</Tag> <Tag>.tgz</Tag> <Tag>.gz</Tag> (≤50&nbsp;MB)
        </Descriptions.Item>
      </Descriptions>

      <section id="lifecycle" className="scroll-mt-24" />
      <Title level={3}>How Fitchfork Runs Main</Title>
      <Timeline
        className="mb-2"
        items={[
          {
            color: 'blue',
            dot: <FileZipOutlined />,
            children: 'Upload Main, Memo, and Makefile archives under Files & Resources.',
          },
          {
            color: 'blue',
            dot: <LoadingOutlined />,
            children:
              'Generate Memo Output: Fitchfork unpacks the three archives and runs your Main for each task to record reference output.',
          },
          {
            color: 'green',
            dot: <PlayCircleOutlined />,
            children:
              'Student run: the same Main + Makefile pipeline runs against the student submission in an isolated container.',
          },
          {
            color: 'gray',
            dot: <DiffOutlined />,
            children:
              'Compare outputs: student results are diffed against the memo output to award marks and detect subsection status.',
          },
        ]}
      />
      <Paragraph>
        If any required archive is missing, Fitchfork stops here with a clear error (e.g. “Main
        files (.zip) not found”). The same validation runs before memo generation <em>and</em>{' '}
        before each student attempt so missing or outdated Main archives are caught early.
      </Paragraph>

      <section id="requirements" className="scroll-mt-24" />
      <Title level={3}>Archive &amp; Entry Requirements</Title>
      <Card>
        <Paragraph className="mb-2">
          The expected entry filename comes from <Text code>config.json → project.language</Text>.
          Use the language picker in Assignment Config to keep Main aligned with the Makefile
          commands you run.
        </Paragraph>
        {/* md+ : normal table */}
        <div className="hidden md:block">
          <Table
            size="small"
            columns={langCols}
            dataSource={langRows}
            pagination={false}
            scroll={{ x: true }}
          />
        </div>

        {/* <md : cards (no extra shadows) */}
        <div className="block md:hidden !space-y-3">
          {langRows.map((r) => (
            <Card
              key={r.key}
              size="small"
              title={<div className="text-base font-semibold truncate">{r.lang}</div>}
            >
              <div className="text-xs uppercase tracking-wide text-gray-500 dark:text-gray-400 mb-1">
                Expected entry file
              </div>
              <div className="text-sm text-gray-900 dark:text-gray-100 mb-2">{r.example}</div>

              <div className="text-xs uppercase tracking-wide text-gray-500 dark:text-gray-400 mb-1">
                Notes
              </div>
              <div className="text-sm text-gray-900 dark:text-gray-100">{r.notes}</div>
            </Card>
          ))}
        </div>
      </Card>
      <Paragraph className="mt-3 mb-0">
        Pack <strong>exactly one entry file at the archive root</strong>. Do not nest it in folders
        or ship helper source files—pull helpers in via the Memo or Makefile instead. The runner
        extracts the archive into the working directory for each task, so any extra files can mask
        student output or break deterministic comparisons.
      </Paragraph>

      <section id="storage" className="scroll-mt-24" />
      <Title level={3}>After You Upload</Title>
      <Paragraph className="mb-0">
        Fitchfork stores the archive alongside the assignment in its <Text code>main/</Text> folder
        and records the upload in the assignment files list. Readiness reports and the Setup
        Checklist look for that stored archive when deciding if an assignment is launch-ready.
        Generate Memo Output and student runs use the same stored file, so you only need to upload
        Main once per update.
      </Paragraph>
      <Paragraph className="mt-3 mb-0">
        Need to replace Main? Uploading a new archive overwrites the stored copy and immediately
        affects the next memo generation or student attempt. Keep the previous version in source
        control in case you need to roll back.
      </Paragraph>

      <section id="tasks" className="scroll-mt-24" />
      <Title level={3}>Task Arguments &amp; CLI</Title>
      <Card>
        <Paragraph className="mb-3">
          Tasks defined under <Text code>Assignments → Tasks</Text> pass a single CLI argument to
          Main. Accept values such as <Text code>task1</Text> and <Text code>task2</Text>, and fall
          back to
          <Text code>task1</Text> if no argument is supplied. This keeps your Makefile targets, task
          commands, and Main in sync.
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
          value={`### Step 1
42
### Step 2
OK
### Step 1
hello
### Step 2
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
