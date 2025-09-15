// src/pages/help/assignments/submissions/HowToSubmitHelp.tsx
import { useEffect, useMemo } from 'react';
import { Typography, Card, Alert, Space, Collapse, Descriptions } from 'antd';
import { useHelpToc } from '@/context/HelpContext';
import { CodeEditor } from '@/components/common';
import { useBreadcrumbContext } from '@/context/BreadcrumbContext';

const { Title, Paragraph, Text } = Typography;

const toc = [
  { key: 'what', href: '#what', title: 'What this page covers' },
  { key: 'where', href: '#where', title: 'Where to submit' },
  { key: 'prep', href: '#prep', title: 'Before you submit (checklist)' },
  { key: 'structure', href: '#structure', title: 'Archive structure (no folders)' },
  { key: 'how', href: '#how', title: 'Submit step-by-step' },
  { key: 'after', href: '#after', title: 'After you submit' },
  { key: 'trouble', href: '#trouble', title: 'Troubleshooting' }, // keep last
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
          <li>
            Submit via the assignment page (top-right <b>Submit</b> button).
          </li>
          <li>
            Your archive must be <b>flat</b> — <b>no nested folders</b>.
          </li>
          <li>
            Include <b>all required source files</b>. Don’t include large binaries.
          </li>
          <li>
            If the assignment is <b>closed</b>, the Submit button will be disabled.
          </li>
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

      <section id="what" className="scroll-mt-24" />
      <Title level={3}>What this page covers</Title>
      <Paragraph className="mb-0">
        For students: how to package your code and submit it. This page also explains the required
        archive layout (<b>no nested directories</b>) and common submission errors.
      </Paragraph>

      <section id="where" className="scroll-mt-24" />
      <Title level={3}>Where to submit</Title>
      <Paragraph className="mb-0">
        Open your assignment. In the <b>top-right</b>, use the <b>Submit</b> button. If the
        assignment isn’t open, the button will be <b>disabled</b> — check the dates or ask your
        lecturer.
      </Paragraph>

      <section id="prep" className="scroll-mt-24" />
      <Title level={3}>Before you submit (checklist)</Title>
      <ul className="list-disc pl-5">
        <li>
          <b>All code included:</b> add every source file needed to build/run.
        </li>
        <li>
          <b>Flat archive:</b> place files at the root — <i>no folders, no nesting</i>.
        </li>
        <li>
          <b>Correct entry file:</b> follow the assignment’s language/filename rules (e.g.,{' '}
          <Text code>Main.java</Text> or <Text code>Main.cpp</Text>).
        </li>
        <li>
          <b>Keep it light:</b> exclude compiled binaries, large datasets, and temporary files.
        </li>
        <li>
          <b>Deterministic output:</b> avoid debug spam or timestamps if your marking compares text.
        </li>
      </ul>

      <section id="structure" className="scroll-mt-24" />
      <Title level={3}>Archive structure (no folders)</Title>
      <Paragraph className="mb-2">
        Submit a single archive (recommended: <Text code>.zip</Text>). Put all required files at the{' '}
        <b>root</b>. Do <u>not</u> include subdirectories.
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
        <li>
          Go to the assignment page and click <b>Submit</b> (top-right).
        </li>
        <li>
          In the submission modal, <b>drag &amp; drop</b> your archive (e.g.,{' '}
          <Text code>submission.zip</Text>).
        </li>
        <li>
          Tick the checkbox to confirm the work is <b>your own</b>.
        </li>
        <li>
          Click <b>Submit</b> to upload. You’ll see status/feedback once the run completes.
        </li>
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
          Your submission is queued, built, and run under the assignment’s limits.
        </Descriptions.Item>
        <Descriptions.Item label="Results">
          View output and marks under the assignment once processing finishes.
        </Descriptions.Item>
        <Descriptions.Item label="Resubmissions">
          If allowed, you can submit again; the assignment’s policy determines which attempt counts.
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
