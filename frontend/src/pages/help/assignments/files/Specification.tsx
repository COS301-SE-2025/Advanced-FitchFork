import { Typography, Card, Descriptions, Tag, Collapse, Tabs, Space, Table, Alert } from 'antd';
import { useEffect, useMemo } from 'react';
import { useHelpToc } from '@/context/HelpContext';
import { CodeEditor } from '@/components/common';
import { useBreadcrumbContext } from '@/context/BreadcrumbContext';

const { Title, Paragraph, Text } = Typography;

const toc = [
  { key: 'what', href: '#what', title: 'What is the Specification ZIP?' },
  { key: 'where', href: '#where', title: 'Where to Upload' },
  { key: 'structure', href: '#structure', title: 'Archive Structure' },
  { key: 'examples', href: '#examples', title: 'Examples' },
  { key: 'plagiarism', href: '#plagiarism', title: 'Plagiarism: Base Files Behavior' },
  { key: 'best', href: '#best', title: 'Best Practices' },
  { key: 'faq', href: '#faq', title: 'FAQ' },
  { key: 'trouble', href: '#trouble', title: 'Troubleshooting' },
];

// Root-only examples (no folders, no git files)
const STRUCTURE_JAVA = `spec.zip
├─ App.java              // skeleton with TODOs
├─ Util.java             // helpers with stubs
├─ spec.pdf              // optional: assignment description
└─ README.md             // optional
`;

const STRUCTURE_CPP = `spec.zip
├─ app.hpp               // function signatures (no body)
├─ app.cpp               // empty or stubbed implementation
├─ util.hpp
├─ util.cpp
├─ spec.pdf              // optional
└─ README.md             // optional
`;

const JAVA_SKELETON = `// App.java (skeleton)
public class App {
    // Students must implement this:
    public static int sum(int[] a) {
        // TODO: implement
        return 0;
    }
}
`;

const CPP_SKELETON_HPP = `// app.hpp (skeleton)
#pragma once
#include <vector>
int sum(const std::vector<int>& a); // declaration only (no body)
`;

const CPP_SKELETON_CPP = `// app.cpp (skeleton)
#include "app.hpp"
int sum(const std::vector<int>& a) {
    // TODO: implement
    return 0;
}
`;

const baseVsStudentCols = [
  { title: 'Example', dataIndex: 'ex', key: 'ex', width: 280 },
  { title: 'In Spec ZIP (Base)?', dataIndex: 'base', key: 'base', width: 180 },
  { title: 'Similarity Flagged?', dataIndex: 'flag', key: 'flag' },
];

const baseVsStudentRows = [
  {
    key: 'r1',
    ex: 'Empty function signatures / headers',
    base: 'Yes (uploaded as base)',
    flag: 'Ignored as boilerplate',
  },
  {
    key: 'r2',
    ex: 'Provided stub with TODO + return 0',
    base: 'Yes (uploaded as base)',
    flag: 'Ignored (identical to skeleton)',
  },
  {
    key: 'r3',
    ex: 'Student’s real implementation body',
    base: 'No (not in skeleton)',
    flag: 'Can be flagged if matches another student',
  },
  {
    key: 'r4',
    ex: 'spec.pdf / README.md',
    base: 'Yes (uploaded as base)',
    flag: 'Ignored (shared docs)',
  },
];

export default function Specification() {
  const { setBreadcrumbLabel } = useBreadcrumbContext();
  const ids = useMemo(() => toc.map((t) => t.href.slice(1)), []);

  useEffect(() => {
    setBreadcrumbLabel('help/assignments/files/specification', 'Specification');
  }, []);

  useHelpToc({
    items: toc,
    ids,
    extra: (
      <Card className="mt-4" size="small" title="Quick facts" bordered>
        <ul className="list-disc pl-5">
          <li>
            Archive for <b>skeleton starter code</b> students receive
          </li>
          <li>
            May include <b>spec.pdf</b> and <b>README.md</b>
          </li>
          <li>
            <b>Root-only files</b> — no folders
          </li>
          <li>
            Also used as <b>base files</b> in plagiarism detection
          </li>
        </ul>
      </Card>
    ),
    onMountScrollToHash: true,
  });

  return (
    <Space direction="vertical" size="small" className="w-full !p-0">
      <Title level={2} className="mb-0">
        Specification
      </Title>

      <section id="what" className="scroll-mt-24" />
      <Title level={3}>What is the Specification ZIP?</Title>
      <Paragraph className="mb-0">
        The <b>Specification</b> ZIP (often <Text code>spec.zip</Text>) bundles the{' '}
        <b>skeleton files</b> you give to students: headers, stubs, scaffolding, and optionally a{' '}
        <Text code>spec.pdf</Text> describing the assignment. Students build on top of this starter.
        The platform also uses this archive as <b>base files</b> for plagiarism detection—shared
        boilerplate is <i>excluded</i> from similarity scoring, while students’ actual
        implementations can still be flagged if they match one another.
      </Paragraph>

      <Descriptions bordered size="middle" column={1} className="mt-3">
        <Descriptions.Item label="Accepted formats">
          <Tag>.zip</Tag> <Tag>.tar</Tag> <Tag>.tgz</Tag> <Tag>.gz</Tag> (max&nbsp;50&nbsp;MB)
        </Descriptions.Item>
        <Descriptions.Item label="Contains">
          Skeleton sources, headers, stub files, optional <Text code>spec.pdf</Text> and{' '}
          <Text code>README.md</Text>. <b>Root-only files — no folders.</b>
        </Descriptions.Item>
        <Descriptions.Item label="Used by">
          <Tag color="geekblue">Students (starter pack)</Tag>{' '}
          <Tag color="purple">Plagiarism (base files)</Tag>
        </Descriptions.Item>
      </Descriptions>

      <section id="where" className="scroll-mt-24" />
      <Title level={3}>Where to Upload</Title>
      <Paragraph className="mb-0">
        Go to <Text code>Assignments → Config → Files</Text> and upload the <b>Specification</b>{' '}
        archive. Students will receive these skeleton files as their starting point, and the same
        archive will be used as <b>base files</b> during similarity checks.
      </Paragraph>

      <section id="structure" className="scroll-mt-24" />
      <Title level={3}>Archive Structure</Title>
      <Paragraph className="mb-0">
        The Specification ZIP can include <b>multiple files</b>, but they must all live at the{' '}
        <b>archive root (no directories)</b>. Keep the code <b>compilable</b> but <b>incomplete</b>{' '}
        (e.g., empty bodies, TODO stubs). Do not include instructor-only artifacts (solutions,
        answer keys, private tests).
      </Paragraph>

      <Card className="mt-2">
        <Tabs
          items={[
            {
              key: 'java',
              label: 'Java',
              children: (
                <CodeEditor
                  title="spec.zip (Java, root-only)"
                  language="plaintext"
                  value={STRUCTURE_JAVA}
                  height={200}
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
                  title="spec.zip (C++, root-only)"
                  language="plaintext"
                  value={STRUCTURE_CPP}
                  height={200}
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
      </Card>

      <section id="examples" className="scroll-mt-24" />
      <Title level={3}>Examples</Title>
      <Card>
        <Tabs
          items={[
            {
              key: 'java',
              label: 'Java skeleton',
              children: (
                <CodeEditor
                  title="App.java (skeleton)"
                  language="java"
                  value={JAVA_SKELETON}
                  height={220}
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
              label: 'C++ skeleton',
              children: (
                <Space direction="vertical" size="small" className="w-full">
                  <CodeEditor
                    title="app.hpp"
                    language="cpp"
                    value={CPP_SKELETON_HPP}
                    height={140}
                    readOnly
                    minimal
                    fitContent
                    showLineNumbers={false}
                    hideCopyButton
                  />
                  <CodeEditor
                    title="app.cpp"
                    language="cpp"
                    value={CPP_SKELETON_CPP}
                    height={160}
                    readOnly
                    minimal
                    fitContent
                    showLineNumbers={false}
                    hideCopyButton
                  />
                </Space>
              ),
            },
          ]}
        />
      </Card>

      <section id="plagiarism" className="scroll-mt-24" />
      <Title level={3}>Plagiarism</Title>

      <Alert
        type="info"
        showIcon
        className="mb-3"
        message="Spec ZIP is treated as base files"
        description={
          <>
            During similarity checks, <b>all files</b> in the Specification ZIP are uploaded as{' '}
            <b>base files</b> (MOSS <Text code>-b</Text>). Shared boilerplate is <b>ignored</b>;
            only student-written code can be flagged.
          </>
        }
      />

      <Table
        size="small"
        pagination={false}
        columns={[
          { title: 'Example', dataIndex: 'ex', key: 'ex', width: 280 },
          { title: 'In Spec (Base)?', dataIndex: 'base', key: 'base', width: 160 },
          { title: 'Flagged?', dataIndex: 'flag', key: 'flag' },
        ]}
        dataSource={[
          { key: '1', ex: 'Function signatures / headers', base: 'Yes', flag: 'Ignored' },
          { key: '2', ex: 'Stub with TODO / return 0', base: 'Yes', flag: 'Ignored' },
          { key: '3', ex: 'Student implementation body', base: 'No', flag: 'Can be flagged' },
          { key: '4', ex: 'spec.pdf / README.md', base: 'Yes', flag: 'Ignored' },
        ]}
      />

      <section id="faq" className="scroll-mt-24" />
      <Title level={3}>FAQ</Title>
      <Collapse
        items={[
          {
            key: 'f1',
            label: 'Will including spec.pdf affect plagiarism reports?',
            children: (
              <Paragraph>
                No. Non-code and shared docs are uploaded as base and are ignored by similarity
                scoring.
              </Paragraph>
            ),
          },
          {
            key: 'f2',
            label: 'What if I update the skeleton after students start?',
            children: (
              <Paragraph>
                Upload the updated Specification. For best results, don’t rename paths; keep the
                skeleton consistent so base matching remains effective.
              </Paragraph>
            ),
          },
          {
            key: 'f3',
            label: 'Do I need to configure filters for the spec files?',
            children: (
              <Paragraph>
                No. The platform always uploads the Specification as <b>base files</b>. Optional
                include/exclude filtering only applies to student submission contents.
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
            label: '“Skeleton code still appears as matched in the report”',
            children: (
              <ul className="list-disc pl-5">
                <li>Confirm you uploaded the correct, current Specification ZIP.</li>
                <li>Ensure students didn’t delete essential skeleton lines/markers.</li>
                <li>Keep filenames identical; renames reduce base matching.</li>
              </ul>
            ),
          },
          {
            key: 't2',
            label: 'Spec ZIP rejected or too large',
            children: (
              <Paragraph>
                Use a supported format (<Text code>.zip</Text>/<Text code>.tar</Text>/
                <Text code>.tgz</Text>/<Text code>.gz</Text>) and keep it ≤ 50&nbsp;MB.
              </Paragraph>
            ),
          },
        ]}
      />
    </Space>
  );
}
