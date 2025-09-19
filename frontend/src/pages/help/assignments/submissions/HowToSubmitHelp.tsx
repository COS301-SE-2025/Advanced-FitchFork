// src/pages/help/assignments/submissions/HowToSubmitHelp.tsx
import { useEffect, useMemo } from 'react';
import { Typography, Card, Alert, Space, Collapse, Descriptions } from 'antd';
import { useHelpToc } from '@/context/HelpContext';
import { CodeEditor } from '@/components/common';
import { useBreadcrumbContext } from '@/context/BreadcrumbContext';

const { Title, Paragraph, Text } = Typography;

const toc = [
  { key: 'overview', href: '#overview', title: 'How Submissions Work' },
  { key: 'where', href: '#where', title: 'Where to submit' },
  { key: 'prep', href: '#prep', title: 'Before you submit' },
  { key: 'structure', href: '#structure', title: 'Archive structure' },
  { key: 'how', href: '#how', title: 'Submit step-by-step' },
  { key: 'after', href: '#after', title: 'After you submit' },
  { key: 'trouble', href: '#trouble', title: 'Troubleshooting' },
];

const FLAT_LAYOUT = `submission.zip
├─ Main.java          // or Main.cpp, per assignment language
├─ App.java           // your source files...
├─ Util.java
└─ README.md          // optional
`;

export default function HowToSubmitHelp() {
  const { setBreadcrumbLabel } = useBreadcrumbContext();
  const ids = useMemo(() => toc.map((t) => t.href.slice(1)), []);

  useEffect(() => {
    setBreadcrumbLabel('help/assignments/submissions/how-to-submit', 'How to Submit');
  }, []);

  useHelpToc({
    items: toc,
    ids,
    extra: (
      <Card className="mt-4" size="small" title="Quick facts" bordered>
        <ul className="list-disc pl-5">
          <li>Submit from the assignment page via the <b>Submit</b> button (top-right).</li>
          <li>Upload a single archive (.zip recommended) with files at the root.</li>
          <li>Include all required sources; leave out binaries and large datasets.</li>
          <li>The button is disabled when the assignment window is closed.</li>
        </ul>
      </Card>
    ),
    onMountScrollToHash: true,
  });

  return (
    <Space direction="vertical" size="small" className="w-full !p-0">
      <Title level={2} className="mb-0">
        How to Submit
      </Title>

      <section id="overview" className="scroll-mt-24" />
      <Title level={3}>How Submissions Work</Title>
      <Paragraph className="mb-0">
        Uploading creates a new attempt that Fitchfork unpacks, builds, and runs using the assignment’s Main/Makefile or
        interpreter. Outputs per task are diffed against Memo Output, then marks and feedback appear on the assignment
        page. Follow the archive rules below so your submission passes validation and runs cleanly on the grader.
      </Paragraph>

      <section id="where" className="scroll-mt-24" />
      <Title level={3}>Where to submit</Title>
      <Paragraph className="mb-0">
        Open your assignment. In the <b>top-right</b>, use the <b>Submit</b> button. If the
        assignment isn’t open, the button will be <b>disabled</b> — check the dates or ask your
        lecturer.
      </Paragraph>

      <section id="prep" className="scroll-mt-24" />
      <Title level={3}>Before you submit</Title>
      <ul className="list-disc pl-5">
        <li><b>Include every required source file</b> so the grader can build and run.</li>
        <li><b>Flatten the archive</b> — no nested directories; files go at the root.</li>
        <li><b>Match the expected entry file</b> (e.g., <Text code>Main.java</Text> or <Text code>Main.cpp</Text>).</li>
        <li><b>Exclude binaries and large data</b> to keep uploads small and clean.</li>
        <li><b>Prefer deterministic output</b> (avoid timestamps/random logs) if marking compares text.</li>
      </ul>

      <section id="structure" className="scroll-mt-24" />
      <Title level={3}>Archive structure (no folders)</Title>
      <Paragraph className="mb-2">
        Submit a single archive (recommended: <Text code>.zip</Text>). Place required files at the <b>root</b> and avoid
        nested folders so the grader can extract them directly.
      </Paragraph>
      <Card>
        <CodeEditor
          title="Example: flat ZIP"
          language="plaintext"
          value={FLAT_LAYOUT}
          height={180}
          readOnly
          minimal
          fitContent
          showLineNumbers={false}
          hideCopyButton
        />
        <ul className="mt-2 mb-0 list-disc pl-5 text-gray-600">
          <li>
            Place all files at the <b>root</b> of the archive.
          </li>
          <li>Do not include folders or nested directories.</li>
        </ul>
      </Card>

      <section id="how" className="scroll-mt-24" />
      <Title level={3}>Submit step-by-step</Title>
      <ol className="list-decimal pl-5">
        <li>Go to the assignment page and click <b>Submit</b> (top-right).</li>
        <li>Drag and drop your archive (e.g., <Text code>submission.zip</Text>) into the modal.</li>
        <li>Confirm the honour code checkbox if prompted.</li>
        <li>Click <b>Submit</b>. The attempt is queued, built, and run; results appear once processing finishes.</li>
      </ol>
      <Alert
        className="mt-2"
        type="info"
        showIcon
        message="Submission window"
        description="If the assignment is closed, you won’t be able to submit. Wait for it to open or contact your lecturer."
      />

      <section id="after" className="scroll-mt-24" />
      <Title level={3}>After you submit</Title>
      <Descriptions bordered size="middle" column={1} className="mt-2">
        <Descriptions.Item label="Status">
          Submissions are queued, built, and run under the assignment’s execution limits. Progress appears on the
          assignment page.
        </Descriptions.Item>
        <Descriptions.Item label="Results">
          View output, marks, and feedback once processing completes. The run history shows each attempt.
        </Descriptions.Item>
        <Descriptions.Item label="Resubmissions">
          If resubmissions are allowed, the assignment’s policy (e.g., “best” or “last”) decides which attempt counts
          toward your mark.
        </Descriptions.Item>
      </Descriptions>

      {/* Troubleshooting LAST */}
      <section id="trouble" className="scroll-mt-24" />
      <Title level={3}>Troubleshooting</Title>
      <Collapse
        items={[
          {
            key: 't1',
            label: '“Submit button is disabled”',
            children: (
              <Paragraph>
                The assignment is likely <b>closed</b>. You can submit only when it’s open.
              </Paragraph>
            ),
          },
          {
            key: 't2',
            label: '“Your archive contains folders” / “Invalid structure”',
            children: (
              <ul className="list-disc pl-5">
                <li>
                  Re-zip your files so everything sits at the <b>root</b> (no nested directories).
                </li>
                <li>
                  Ensure required entry file names follow the spec (e.g.,{' '}
                  <Text code>Main.java</Text>).
                </li>
              </ul>
            ),
          },
          {
            key: 't3',
            label: '“Archive rejected or too large after unzip”',
            children: (
              <ul className="list-disc pl-5">
                <li>Remove compiled binaries, large assets, and temporary files.</li>
                <li>Only include the source files needed to build/run.</li>
              </ul>
            ),
          },
          {
            key: 't4',
            label: '“No output / wrong output”',
            children: (
              <ul className="list-disc pl-5">
                <li>
                  Verify your program prints the expected text to <b>stdout</b>.
                </li>
                <li>Match the task argument and labels exactly if your assignment uses them.</li>
              </ul>
            ),
          },
        ]}
      />
    </Space>
  );
}
