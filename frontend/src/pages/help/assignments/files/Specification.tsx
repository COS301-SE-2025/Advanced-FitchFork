import { Typography, Card, Descriptions, Tag, Collapse, Tabs, Space, Table, Alert } from 'antd';
import { useEffect, useMemo } from 'react';
import { useHelpToc } from '@/context/HelpContext';
import { CodeEditor, GatlamLink } from '@/components/common';
import { useBreadcrumbContext } from '@/context/BreadcrumbContext';
import { useViewSlot } from '@/context/ViewSlotContext';

const { Title, Paragraph, Text } = Typography;

const toc = [
  { key: 'overview', href: '#overview', title: 'Why the Specification Archive Matters' },
  { key: 'lifecycle', href: '#lifecycle', title: 'How Fitchfork Uses Spec Files' },
  { key: 'requirements', href: '#requirements', title: 'Archive Requirements' },
  { key: 'storage', href: '#storage', title: 'After You Upload' },
  { key: 'structure', href: '#structure', title: 'Archive Structure' },
  { key: 'examples', href: '#examples', title: 'Examples' },
  { key: 'plagiarism', href: '#plagiarism', title: 'Plagiarism Base Files' },
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

// Plagiarism table (with mobile card alternative)
const baseCols = [
  { title: 'Example', dataIndex: 'ex', key: 'ex', width: 280 },
  { title: 'In Spec (Base)?', dataIndex: 'base', key: 'base', width: 160 },
  { title: 'Flagged?', dataIndex: 'flag', key: 'flag' },
];

const baseRows = [
  { key: '1', ex: 'Function signatures / headers', base: 'Yes', flag: 'Ignored' },
  { key: '2', ex: 'Stub with TODO / return 0', base: 'Yes', flag: 'Ignored' },
  { key: '3', ex: 'Student implementation body', base: 'No', flag: 'Can be flagged' },
  { key: '4', ex: 'spec.pdf / README.md', base: 'Yes', flag: 'Ignored' },
];

export default function Specification() {
  const { setBreadcrumbLabel } = useBreadcrumbContext();
  const ids = useMemo(() => toc.map((t) => t.href.slice(1)), []);
  const { setValue, setBackTo } = useViewSlot();

  useEffect(() => {
    setValue(
      <Typography.Text className="text-base font-medium text-gray-900 dark:text-gray-100 truncate">
        Specification
      </Typography.Text>,
    );
    setBackTo('/help');
  }, []);

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
            Upload under <Text code>Assignments → Config → Files</Text>
          </li>
          <li>Archive contains skeleton starter code plus optional docs (spec.pdf, README)</li>
          <li>Root-only files (no folders) so the runner and MOSS can ingest them</li>
          <li>
            Everything in the archive is sent to MOSS as base files to suppress boilerplate matches
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

      <section id="overview" className="scroll-mt-24" />
      <Title level={3}>Why the Specification Archive Matters</Title>
      <Paragraph className="mb-0">
        The <strong>Specification</strong> archive (usually <Text code>spec.zip</Text>) bundles the
        skeleton students inherit— headers, stubs, scaffolding, and optional docs. Fitchfork also
        passes these files to MOSS as base files, so shared boilerplate is ignored while
        student-written code can still be flagged. Keeping this archive clean and consistent
        protects both the student starting experience and plagiarism accuracy.
      </Paragraph>
      <Paragraph className="mt-3 mb-0">
        Upload the archive under <Text code>Assignments → Config → Files</Text>. Lecturers/assistant
        lecturers own the upload. Manual submissions require it; in{' '}
        <GatlamLink tone="inherit" icon={false} underline={false}>
          GATLAM
        </GatlamLink>{' '}
        or interpreter workflows
        it still serves as the canonical starter pack and base-file input.
      </Paragraph>

      <Descriptions bordered size="middle" column={1} className="mt-3">
        <Descriptions.Item label="Formats">
          <Tag>.zip</Tag> <Tag>.tar</Tag> <Tag>.tgz</Tag> <Tag>.gz</Tag> (≤50&nbsp;MB recommended)
        </Descriptions.Item>
        <Descriptions.Item label="Contents">
          Skeleton sources, headers, stubbed implementations, optional <Text code>spec.pdf</Text>/
          <Text code>README.md</Text>. All files must sit at the archive root.
        </Descriptions.Item>
        <Descriptions.Item label="Used by">
          <Tag color="geekblue">Students</Tag> (starter code) and{' '}
          <Tag color="purple">Plagiarism</Tag> (base files).
        </Descriptions.Item>
      </Descriptions>

      <section id="lifecycle" className="scroll-mt-24" />
      <Title level={3}>How Fitchfork Uses Spec Files</Title>
      <Card>
        <Tabs
          items={[
            {
              key: 'pipeline',
              label: 'Pipeline',
              children: (
                <ul className="list-disc pl-5">
                  <li>
                    When students clone the assignment, they receive the Specification archive as
                    their starter code.
                  </li>
                  <li>
                    During memo generation and student runs, the archive is extracted alongside Main
                    and Makefile so the same stubs are available.
                  </li>
                  <li>
                    When you run MOSS, all files in the archive are uploaded as base files; MOSS
                    ignores exact matches from this skeleton.
                  </li>
                </ul>
              ),
            },
            {
              key: 'storage',
              label: 'Storage & readiness',
              children: (
                <Paragraph className="mb-0">
                  The archive is stored in <Text code>spec/</Text>. Readiness reports and the Setup
                  Checklist check for its presence. Uploading a new spec overwrites the stored copy
                  and is picked up on the next student download or MOSS run, so keep versions under
                  source control.
                </Paragraph>
              ),
            },
          ]}
        />
      </Card>

      <section id="requirements" className="scroll-mt-24" />
      <Title level={3}>Archive Requirements</Title>
      <ul className="list-disc pl-5">
        <li>Files must live at the archive root (no nested directories).</li>
        <li>
          Provide compilable headers/stubs but leave implementations incomplete (e.g., TODOs, return
          0).
        </li>
        <li>Do not include instructor-only assets like solutions, hidden tests, or credentials.</li>
        <li>
          Optional docs (spec.pdf, README.md) are welcome; MOSS treats them as base files too.
        </li>
      </ul>

      <section id="structure" className="scroll-mt-24" />
      <Title level={3}>Archive Structure</Title>
      <Card>
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
      <Title level={3}>Plagiarism Base Files</Title>

      <Alert
        type="info"
        showIcon
        className="mb-3"
        message="Spec ZIP is treated as base files"
        description={
          <>
            During MOSS runs, every file in the Specification archive is uploaded as a base file (
            <Text code>-b</Text>). Shared boilerplate is ignored, while code outside the skeleton
            remains eligible for similarity matches.
          </>
        }
      />

      {/* Desktop table */}
      <div className="hidden md:block">
        <Table
          size="small"
          pagination={false}
          columns={baseCols}
          dataSource={baseRows}
          scroll={{ x: true }}
        />
      </div>

      {/* Mobile card alternative */}
      <div className="block md:hidden mt-2 !space-y-3">
        {baseRows.map((r) => (
          <Card
            key={r.key}
            size="small"
            title={<div className="text-base font-semibold truncate">{r.ex}</div>}
          >
            <div className="text-sm">
              <div className="text-gray-500 dark:text-gray-400 uppercase tracking-wide text-xs mb-1">
                In Spec (Base)?
              </div>
              <div className="text-gray-900 dark:text-gray-100">{r.base}</div>
              <div className="text-gray-500 dark:text-gray-400 uppercase tracking-wide text-xs mt-2 mb-1">
                Flagged?
              </div>
              <div className="text-gray-900 dark:text-gray-100">{r.flag}</div>
            </div>
          </Card>
        ))}
      </div>

      <section id="best" className="scroll-mt-24" />
      <Title level={3}>Best Practices</Title>
      <ul className="list-disc pl-5">
        <li>
          Keep all files at the archive root; nested folders are ignored and can break ingestion.
        </li>
        <li>Provide only skeleton/stub code — never include full solutions or private tests.</li>
        <li>Avoid large binaries; keep the ZIP under ~50&nbsp;MB.</li>
        <li>Keep filenames stable to maximize base-file matching in MOSS.</li>
      </ul>

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
            label: '“Specification archive missing or empty”',
            children: (
              <Paragraph>
                Upload a supported archive to the Specification slot and keep files at the root.
                Readiness checks and memo generation will continue to fail until the archive
                contains at least one file.
              </Paragraph>
            ),
          },
          {
            key: 't2',
            label: 'Skeleton code still appears in plagiarism matches',
            children: (
              <ul className="list-disc pl-5">
                <li>Verify the uploaded Specification matches the skeleton students were given.</li>
                <li>
                  Keep filenames consistent; renaming files after students download can prevent base
                  matching.
                </li>
                <li>
                  Ensure students retain required markers (e.g., delimiter prints) so shared
                  sections align.
                </li>
              </ul>
            ),
          },
          {
            key: 't3',
            label: 'Archive rejected or too large',
            children: (
              <Paragraph>
                Use .zip/.tar/.tgz/.gz and keep the archive under ~50&nbsp;MB. Remove compiled
                artefacts or large binaries from the skeleton before uploading.
              </Paragraph>
            ),
          },
        ]}
      />
    </Space>
  );
}
