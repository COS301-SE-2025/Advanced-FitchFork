// src/pages/help/assignments/files/MemoFiles.tsx
import { Typography, Card, Descriptions, Tag, Collapse, Tabs, Timeline, Space, Alert } from 'antd';
import { FileZipOutlined, PlayCircleOutlined, DiffOutlined } from '@ant-design/icons';
import { useEffect, useMemo } from 'react';
import { useHelpToc } from '@/context/HelpContext';
import { CodeEditor, GatlamLink } from '@/components/common';
import { useBreadcrumbContext } from '@/context/BreadcrumbContext';
import { useViewSlot } from '@/context/ViewSlotContext';

const { Title, Paragraph, Text } = Typography;

const toc = [
  { key: 'overview', href: '#overview', title: 'Why Memo Files Matter' },
  { key: 'lifecycle', href: '#lifecycle', title: 'How Fitchfork Uses Memo Files' },
  { key: 'requirements', href: '#requirements', title: 'Archive Requirements' },
  { key: 'storage', href: '#storage', title: 'After You Upload' },
  { key: 'layout', href: '#layout', title: 'Archive Layout & Examples' },
  { key: 'best', href: '#best', title: 'Best Practices' },
  { key: 'trouble', href: '#trouble', title: 'Troubleshooting' },
];

// Archive layout previews
const ARCHIVE_JAVA = `memo.zip
├─ Solution.java
└─ Utils.java
`;

const ARCHIVE_CPP = `memo.zip
├─ solution.cpp
├─ utils.hpp
└─ utils.cpp
`;

// Minimal individual files (no Main included)
const JAVA_SOLUTION = `// Solution.java
public class Solution {
    public static int add(int a, int b) { return a + b; }
    public static String greet(String name) { return "Hello, " + name; }
}
`;

const JAVA_UTILS = `// Utils.java
class Utils {
    public static int clamp(int x, int lo, int hi) {
        return Math.max(lo, Math.min(hi, x));
    }
}
`;

const CPP_SOLUTION = `// solution.cpp
#include <string>
#include <algorithm>

int add(int a, int b) { return a + b; }

std::string greet(const std::string& name) {
    return std::string("Hello, ") + name;
}
`;

const CPP_UTILS_HPP = `// utils.hpp
#pragma once
#include <algorithm>

inline int clamp(int x, int lo, int hi) {
    return std::max(lo, std::min(hi, x));
}
`;

const CPP_UTILS_CPP = `// utils.cpp
#include "utils.hpp"
// (Add larger helpers here if needed)
`;

export default function MemoFiles() {
  const { setBreadcrumbLabel } = useBreadcrumbContext();
  const ids = useMemo(() => toc.map((t) => t.href.slice(1)), []);
  const { setValue, setBackTo } = useViewSlot();

  useEffect(() => {
    setValue(
      <Typography.Text className="text-base font-medium text-gray-900 dark:text-gray-100 truncate">
        Memo Files
      </Typography.Text>,
    );
    setBackTo('/help');
  }, []);

  useEffect(() => {
    setBreadcrumbLabel('help/assignments/files/memo-files', 'Memo Files');
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
          <li>Archive is the official solution; place source files at root (no folders)</li>
          <li>Exclude Main; Main has its own archive</li>
          <li>Students never see memo code—only outputs generated from it</li>
        </ul>
      </Card>
    ),
    onMountScrollToHash: true,
  });

  return (
    <Space direction="vertical" size="small" className="w-full !p-0">
      <Title level={2} className="mb-0">
        Memo Files
      </Title>

      <section id="overview" className="scroll-mt-24" />
      <Title level={3}>Why Memo Files Matter</Title>
      <Paragraph className="mb-0">
        The <strong>Memo</strong> archive is your authoritative solution. Fitchfork compiles and
        runs it with Main and the Makefile to produce the <strong>Memo Output</strong> that anchors
        every comparison. When students submit, their code is judged against the behaviour captured
        from these memo classes, so keeping the archive accurate and deterministic is essential.
      </Paragraph>
      <Paragraph className="mt-3 mb-0">
        Upload memo files under <Text code>Assignments → Config → Files</Text>. Lecturers (or
        assistant lecturers) own this upload. In Manual mode it is required; in{' '}
        <GatlamLink tone="inherit" icon={false} underline={false}>
          GATLAM
        </GatlamLink>{' '}
        mode memo
        files still provide reference behaviour even though the Interpreter handles code generation.
      </Paragraph>

      <Alert
        type="warning"
        showIcon
        className="mt-3"
        message="Keep Main separate"
        description="The memo archive must not contain Main.java/Main.cpp. Main belongs to the Main archive and is uploaded separately."
      />

      <Descriptions bordered size="middle" column={1} className="mt-3">
        <Descriptions.Item label="Formats">
          <Tag>.zip</Tag> <Tag>.tar</Tag> <Tag>.tgz</Tag> <Tag>.gz</Tag> (≤50&nbsp;MB recommended)
        </Descriptions.Item>
        <Descriptions.Item label="Structure">
          Source files only, placed at the archive root (no directories, binaries, or build
          artefacts).
        </Descriptions.Item>
        <Descriptions.Item label="Visibility">
          Students never access memo code; they only see marks and feedback derived from the
          captured outputs.
        </Descriptions.Item>
      </Descriptions>

      <section id="lifecycle" className="scroll-mt-24" />
      <Title level={3}>How Fitchfork Uses Memo Files</Title>
      <Timeline
        className="mb-2"
        items={[
          {
            color: 'blue',
            dot: <FileZipOutlined />,
            children:
              'Upload Main, Makefile, and Memo archives. Fitchfork checks that each slot contains a supported archive.',
          },
          {
            color: 'green',
            dot: <PlayCircleOutlined />,
            children:
              'Generate Memo Output: the runner compiles your memo classes via the Makefile and executes Main to capture reference output per task.',
          },
          {
            color: 'gray',
            dot: <DiffOutlined />,
            children:
              'Student submissions run through the same pipeline; their output is diffed against Memo Output using your Mark Allocator.',
          },
        ]}
      />
      <Paragraph className="mt-2 mb-0">
        In practice, <b>Main</b> is the tiny runner that <i>calls your memo code</i> for each Task
        and prints the results. When you generate Memo Output, the platform builds/runs via your
        <b> Makefile</b>, executes <b>Main</b> with the memo classes, and saves one text result per
        task. For details about those files and naming, see{' '}
        <a href="/help/assignments/memo-output">Memo Output</a>.
      </Paragraph>

      <section id="storage" className="scroll-mt-24" />
      <Title level={3}>After You Upload</Title>
      <Paragraph className="mb-2">
        Memo files are stored in the assignment’s <Text code>memo/</Text> folder and recorded in the
        assignment files list. Readiness checks and the Setup Checklist look for a memo archive
        before marking the assignment ready. Uploading a new archive overwrites the stored copy and
        affects the next memo generation or student run, so keep earlier versions in source control
        if you might need to roll back.
      </Paragraph>
      <Descriptions bordered size="middle" column={1}>
        <Descriptions.Item label="Validation">
          Missing or empty memo directories trigger “Required memo directory is missing or empty” or
          “Memo archive (.zip) not found” during memo generation and student attempts.
        </Descriptions.Item>
        <Descriptions.Item label="Regeneration">
          After updating memo code, regenerate Memo Output so comparison files reflect the new
          behaviour.
        </Descriptions.Item>
      </Descriptions>

      <section id="layout" className="scroll-mt-24" />
      <Title level={3}>Archive Layout &amp; Examples</Title>
      <Card>
        <Tabs
          items={[
            {
              key: 'java',
              label: 'Java',
              children: (
                <Space direction="vertical" size="small" className="w-full">
                  <CodeEditor
                    title="Archive layout (Java)"
                    language="plaintext"
                    value={ARCHIVE_JAVA}
                    height={160}
                    readOnly
                    minimal
                    fitContent
                    showLineNumbers={false}
                    hideCopyButton
                  />
                  <CodeEditor
                    title="Solution.java"
                    language="java"
                    value={JAVA_SOLUTION}
                    height={220}
                    readOnly
                    minimal
                    fitContent
                    showLineNumbers={false}
                    hideCopyButton
                  />
                  <CodeEditor
                    title="Utils.java"
                    language="java"
                    value={JAVA_UTILS}
                    height={180}
                    readOnly
                    minimal
                    fitContent
                    showLineNumbers={false}
                    hideCopyButton
                  />
                </Space>
              ),
            },
            {
              key: 'cpp',
              label: 'C++',
              children: (
                <Space direction="vertical" size="small" className="w-full">
                  <CodeEditor
                    title="Archive layout (C++)"
                    language="plaintext"
                    value={ARCHIVE_CPP}
                    height={160}
                    readOnly
                    minimal
                    fitContent
                    showLineNumbers={false}
                    hideCopyButton
                  />
                  <CodeEditor
                    title="solution.cpp"
                    language="cpp"
                    value={CPP_SOLUTION}
                    height={220}
                    readOnly
                    minimal
                    fitContent
                    showLineNumbers={false}
                    hideCopyButton
                  />
                  <CodeEditor
                    title="utils.hpp"
                    language="cpp"
                    value={CPP_UTILS_HPP}
                    height={180}
                    readOnly
                    minimal
                    fitContent
                    showLineNumbers={false}
                    hideCopyButton
                  />
                  <CodeEditor
                    title="utils.cpp"
                    language="cpp"
                    value={CPP_UTILS_CPP}
                    height={120}
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
        <ul className="list-disc pl-5 mt-3">
          <li>All files must be at the archive root (no subfolders).</li>
          <li>
            Do <b>not</b> include Main.java/Main.cpp in this archive.
          </li>
        </ul>
      </Card>

      <section id="best" className="scroll-mt-24" />
      <Title level={3}>Best Practices</Title>
      <ul className="list-disc pl-5">
        <li>
          <b>Root-only</b>: keep source files at the archive root. Nested folders are ignored when
          memo generation runs.
        </li>
        <li>
          <b>Exclude Main</b>: Main belongs in its own archive. Memo files should compile cleanly
          without it.
        </li>
        <li>
          <b>Deterministic behaviour</b>: avoid timestamps, randomness, or environment-dependent
          output.
        </li>
        <li>
          <b>Lean dependencies</b>: vendor helper code in the archive; do not rely on network
          downloads.
        </li>
        <li>
          <b>Short, consistent output</b>: print only what your allocators expect so diffs stay
          stable.
        </li>
      </ul>

      <section id="trouble" className="scroll-mt-24" />
      <Title level={3}>Troubleshooting</Title>
      <Collapse
        items={[
          {
            key: 't1',
            label: '“Required memo directory is missing or empty”',
            children: (
              <Paragraph>
                Upload a supported archive to the Memo slot and ensure it contains at least one
                source file at the root. Generating Memo Output immediately checks for this and
                fails fast if the archive is missing.
              </Paragraph>
            ),
          },
          {
            key: 't2',
            label: 'Memo generation fails inside your code',
            children: (
              <ul className="list-disc pl-5">
                <li>
                  Verify each Task command in <Text>Assignments → Tasks</Text> matches the targets
                  your Makefile exposes.
                </li>
                <li>
                  Ensure memo files compile cleanly when combined with Main and Makefile locally.
                </li>
                <li>
                  Adjust execution limits under Assignment Config → Execution if builds legitimately
                  need more time or memory.
                </li>
              </ul>
            ),
          },
          {
            key: 't3',
            label: 'Outputs differ between memo runs and student runs',
            children: (
              <Paragraph>
                Remove nondeterminism—avoid random seeds, system time, or environment-dependent
                paths. Memo Output must stay stable for comparisons to pass.
              </Paragraph>
            ),
          },
          {
            key: 't4',
            label: 'Students score unexpectedly low',
            children: (
              <ul className="list-disc pl-5">
                <li>
                  Inspect Memo Output under <Text code>memo_output/</Text> to confirm formatting
                  matches your expectations.
                </li>
                <li>
                  Check the Mark Allocator to ensure subsections and point weights align with what
                  Main prints.
                </li>
                <li>
                  Consider a more tolerant comparator if whitespace or ordering differences are
                  acceptable.
                </li>
              </ul>
            ),
          },
        ]}
      />
    </Space>
  );
}
