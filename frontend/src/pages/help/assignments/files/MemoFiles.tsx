// src/pages/help/assignments/files/MemoFiles.tsx
import {
  Typography,
  Card,
  Descriptions,
  Tag,
  Collapse,
  Tabs,
  Timeline,
  Space,
  Steps,
  Alert,
} from 'antd';
import { FileZipOutlined, PlayCircleOutlined, DiffOutlined } from '@ant-design/icons';
import { useEffect, useMemo } from 'react';
import { useHelpToc } from '@/context/HelpContext';
import { CodeEditor } from '@/components/common';
import { useBreadcrumbContext } from '@/context/BreadcrumbContext';

const { Title, Paragraph, Text } = Typography;

const toc = [
  { key: 'what', href: '#what', title: 'What are Memo Files?' },
  { key: 'role', href: '#role', title: 'How They’re Used in Marking' },
  { key: 'upload', href: '#upload', title: 'Where to Upload' },
  { key: 'layout', href: '#layout', title: 'Archive & Examples' },
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

  useEffect(() => {
    setBreadcrumbLabel('help/assignments/files/memo-files', 'Memo Files');
  }, []);

  useHelpToc({
    items: toc,
    ids,
    extra: (
      <Card className="mt-4" size="small" title="Quick facts" bordered>
        <ul className="list-disc pl-5">
          <li>Archive is the official solution (reference implementation)</li>
          <li>
            <b>Root-only files</b> — no folders
          </li>
          <li>
            <b>Do not include Main</b> (Main lives in its own archive)
          </li>
          <li>Students don’t see memo code; only results are used for marking</li>
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

      <section id="what" className="scroll-mt-24" />
      <Title level={3}>What are Memo Files?</Title>
      <Paragraph className="mb-0">
        <b>Memo Files</b> are the fully implemented classes and helpers that define the correct
        behaviour. The platform runs this archive alongside <b>Main</b> and <b>Makefile</b> to
        produce reference results for marking.
      </Paragraph>
      <Alert
        type="warning"
        showIcon
        className="mt-3"
        message="Do not include Main here"
        description="The memo archive must not contain Main.java/Main.cpp. Main belongs to the Main archive."
      />
      <Descriptions bordered size="middle" column={1} className="mt-3">
        <Descriptions.Item label="Accepted formats">
          <Tag>.zip</Tag> <Tag>.tar</Tag> <Tag>.tgz</Tag> <Tag>.gz</Tag> (≤ 50MB recommended)
        </Descriptions.Item>
        <Descriptions.Item label="Must contain">
          <b>Only files at the root</b> (no directories). Place all Java/C++ source files directly
          in the archive root.
        </Descriptions.Item>
        <Descriptions.Item label="Visibility">
          Students never see memo code; they see marks and feedback derived from it.
        </Descriptions.Item>
      </Descriptions>

      <section id="role" className="scroll-mt-24" />
      <Title level={3}>How They’re Used in Marking</Title>
      <Timeline
        className="mb-2"
        items={[
          {
            color: 'blue',
            dot: <FileZipOutlined />,
            children: (
              <>
                Upload <b>Main</b>, <b>Makefile</b>, and <b>Memo</b> archives.
              </>
            ),
          },
          {
            color: 'green',
            dot: <PlayCircleOutlined />,
            children: <>Generate Memo Output for each Task.</>,
          },
          {
            color: 'gray',
            dot: <DiffOutlined />,
            children: <>Compare student outputs to the Memo Output using your Mark Allocator.</>,
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

      <section id="upload" className="scroll-mt-24" />
      <Title level={3}>Where to Upload</Title>
      <Card>
        <Steps
          direction="vertical"
          items={[
            {
              title: 'Upload memo archive',
              description: (
                <>
                  Go to <Text code>Assignments → Config → Files</Text> and upload the <b>Memo</b>{' '}
                  archive.
                </>
              ),
            },
          ]}
        />
      </Card>

      <section id="layout" className="scroll-mt-24" />
      <Title level={3}>Archive & Examples</Title>
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
          <b>Root-only</b>: keep all source files at the archive root (no directories).
        </li>
        <li>
          <b>No Main here</b>: Main belongs in the Main archive.
        </li>
        <li>
          <b>Deterministic behaviour</b>: avoid timestamps, randomness, or machine paths.
        </li>
        <li>
          <b>Quiet output</b>: print only what’s relevant for marking.
        </li>
        <li>
          <b>Match Tasks</b>: ensure behaviour aligns with your Task commands.
        </li>
      </ul>

      <section id="trouble" className="scroll-mt-24" />
      <Title level={3}>Troubleshooting</Title>
      <Collapse
        items={[
          {
            key: 't1',
            label: 'Memo generation fails',
            children: (
              <ul className="list-disc pl-5">
                <li>
                  Verify each Task’s command under <Text>Assignments → Tasks</Text>.
                </li>
                <li>Ensure files are at the archive root; remove folders.</li>
                <li>Adjust time/memory/CPU in Execution settings if needed.</li>
              </ul>
            ),
          },
          {
            key: 't2',
            label: 'Outputs change between runs',
            children: (
              <Paragraph>
                Remove non-determinism (RNG, time, absolute paths). Stable outputs are required.
              </Paragraph>
            ),
          },
          {
            key: 't3',
            label: 'Students score unexpectedly low',
            children: (
              <ul className="list-disc pl-5">
                <li>Re-check the Memo Output text formatting.</li>
                <li>
                  Confirm sections/subsections and points in the allocator match expectations.
                </li>
                <li>Use a more forgiving comparator where appropriate.</li>
              </ul>
            ),
          },
        ]}
      />
    </Space>
  );
}
