import { useEffect, useMemo } from 'react';
import { Typography, Card, Space, Collapse, Table, Tag } from 'antd';
import { useHelpToc } from '@/context/HelpContext';
import { CodeEditor } from '@/components/common';
import { useBreadcrumbContext } from '@/context/BreadcrumbContext';
import { useViewSlot } from '@/context/ViewSlotContext';

const { Title, Paragraph, Text } = Typography;

const toc = [
  { key: 'what', href: '#what', title: 'What does Marking control?' },
  { key: 'options', href: '#options', title: 'Options & defaults' },
  { key: 'schemes', href: '#schemes', title: 'Schemes at a glance' },
  { key: 'disallowed', href: '#disallowed', title: 'Disallowed code' },
  { key: 'tips', href: '#tips', title: 'Tips' },
  { key: 'json', href: '#json', title: 'Raw config (JSON)' },
  { key: 'trouble', href: '#trouble', title: 'Troubleshooting' }, // keep last
];

type Row = {
  key: string;
  setting: string;
  meaning: string;
  options?: string[];
  def: string;
};

const optionCols = [
  { title: 'Setting', dataIndex: 'setting', key: 'setting', width: 240 },
  { title: 'What it does', dataIndex: 'meaning', key: 'meaning' },
  {
    title: 'Options',
    dataIndex: 'options',
    key: 'options',
    width: 280,
    render: (opts?: string[]) =>
      opts && opts.length ? (
        <div className="flex flex-wrap gap-1">
          {opts.map((o) => (
            <Tag key={o}>{o}</Tag>
          ))}
        </div>
      ) : (
        <span className="text-gray-500">—</span>
      ),
  },
  { title: 'Default', dataIndex: 'def', key: 'def', width: 140 },
];

const optionRows: Row[] = [
  {
    key: 'scheme',
    setting: 'Marking scheme',
    meaning: 'How Student Output is compared to Memo Output.',
    options: ['Exact', 'Percentage', 'Regex'],
    def: 'Exact',
  },
  {
    key: 'feedback',
    setting: 'Feedback release',
    meaning: 'How feedback is produced/released to students.',
    options: ['Auto', 'Manual', 'AI'],
    def: 'Auto',
  },
  {
    key: 'delimiter',
    setting: 'Subsection delimiter',
    meaning:
      'Marker your Main prints to split a task into subsections. The text after it becomes the subsection name.',
    options: undefined,
    def: '###',
  },
  {
    key: 'policy',
    setting: 'Grading policy',
    meaning: 'Which submission counts for marks.',
    options: ['Best (highest score)', 'Last (most recent)'],
    def: 'Last',
  },
  {
    key: 'limit_attempts',
    setting: 'Limit attempts',
    meaning:
      'If ON, attempts are capped by “Max attempts”. Practice attempts don’t consume the cap.',
    options: ['On', 'Off'],
    def: 'Off',
  },
  {
    key: 'max_attempts',
    setting: 'Max attempts',
    meaning: 'Maximum graded attempts per student (only when “Limit attempts” is ON).',
    options: undefined,
    def: '10',
  },
  {
    key: 'pass_mark',
    setting: 'Pass mark',
    meaning: 'Minimum percentage required to pass the assignment.',
    options: undefined,
    def: '50%',
  },
  {
    key: 'practice',
    setting: 'Allow practice submissions',
    meaning: 'Lets students make “practice” runs that don’t use up the graded attempt limit.',
    options: ['On', 'Off'],
    def: 'Off',
  },
  {
    key: 'disallowed',
    setting: 'Disallowed code',
    meaning:
      'Blocklist of imports/headers/APIs students may not use. Submissions that contain any entry are flagged.',
    options: ['e.g., #include <bits/stdc++.h>', 'import java.net.*', 'Runtime.getRuntime()'],
    def: 'None',
  },
];

// JSON reference (shown at the bottom)
const DEFAULTS_JSON = `{
  "marking": {
    "marking_scheme": "exact",
    "feedback_scheme": "auto",
    "deliminator": "###",
    "grading_policy": "last",
    "max_attempts": 10,
    "limit_attempts": false,
    "pass_mark": 50,
    "allow_practice_submissions": false,
    "dissalowed_code": []
  }
}`;

const COMMON_VARIANT_JSON = `{
  "marking": {
    "marking_scheme": "percentage",
    "feedback_scheme": "manual",
    "deliminator": "###",
    "grading_policy": "best",
    "max_attempts": 3,
    "limit_attempts": true,
    "pass_mark": 60,
    "allow_practice_submissions": true,
    "dissalowed_code": [
      "#include <bits/stdc++.h>",
      "import java.net.*",
      "Runtime.getRuntime()"
    ]
  }
}`;

export default function MarkingHelp() {
  const { setBreadcrumbLabel } = useBreadcrumbContext();
  const ids = useMemo(() => toc.map((t) => t.href.slice(1)), []);
  const { setValue, setBackTo } = useViewSlot();

  useEffect(() => {
    setValue(
      <Typography.Text className="text-base font-medium text-gray-900 dark:text-gray-100 truncate">
        Marking Config
      </Typography.Text>,
    );
    setBackTo('/help');
  }, []);

  useEffect(() => {
    setBreadcrumbLabel('help/assignments/config/marking', 'Marking');
  }, []);

  useHelpToc({
    items: toc,
    ids,
    extra: (
      <Card className="mt-4" size="small" title="Quick facts" bordered>
        <ul className="list-disc pl-5">
          <li>
            Controls comparison method, feedback flow, attempts, pass mark, and disallowed code.
          </li>
          <li>
            Delimiter must match what your <b>Main</b> prints to label subsections.
          </li>
          <li>Practice submissions never consume graded attempts.</li>
        </ul>
      </Card>
    ),
    onMountScrollToHash: true,
  });

  return (
    <Space direction="vertical" size="small" className="w-full !p-0">
      <Title level={2} className="mb-0">
        Marking
      </Title>

      <section id="what" className="scroll-mt-24" />
      <Title level={3}>What does Marking control?</Title>
      <Paragraph className="mb-0">
        Marking decides <b>how outputs are judged</b>, <b>when feedback appears</b>, which attempt
        counts, and whether attempts are limited. The delimiter ties into your Main’s output to
        split tasks into subsections for clearer results.
      </Paragraph>

      <section id="options" className="scroll-mt-24" />
      <Title level={3}>Options & defaults</Title>

      {/* md+ : normal table */}
      <div className="hidden md:block">
        <Table
          className="mt-2"
          size="small"
          columns={optionCols as any}
          dataSource={optionRows}
          pagination={false}
          scroll={{ x: true }}
        />
      </div>

      {/* <md : cards (no extra shadows) */}
      <div className="block md:hidden mt-2 !space-y-3">
        {optionRows.map((r) => (
          <Card
            key={r.key}
            size="small"
            title={<div className="text-base font-semibold truncate">{r.setting}</div>}
          >
            <div className="text-xs uppercase tracking-wide text-gray-500 dark:text-gray-400 mb-1">
              What it does
            </div>
            <div className="text-sm text-gray-900 dark:text-gray-100 mb-2">{r.meaning}</div>

            <div className="text-xs uppercase tracking-wide text-gray-500 dark:text-gray-400 mb-1">
              Options
            </div>
            <div className="text-sm text-gray-900 dark:text-gray-100 mb-2">
              {Array.isArray(r.options) && r.options.length ? (
                <div className="flex flex-wrap gap-1">
                  {r.options.map((o) => (
                    <Tag key={o}>{o}</Tag>
                  ))}
                </div>
              ) : (
                <span className="text-gray-500">—</span>
              )}
            </div>

            <div className="text-xs uppercase tracking-wide text-gray-500 dark:text-gray-400 mb-1">
              Default
            </div>
            <div className="text-sm text-gray-900 dark:text-gray-100">{r.def}</div>
          </Card>
        ))}
      </div>

      <section id="schemes" className="scroll-mt-24" />
      <Title level={3}>Schemes at a glance</Title>
      <Card>
        <ul className="list-disc pl-5">
          <li>
            <b>Marking scheme</b>: <Tag>Exact</Tag> strict text match · <Tag>Percentage</Tag>{' '}
            partial credit by similarity · <Tag>Regex</Tag> pattern-based (advanced).
          </li>
          <li>
            <b>Feedback release</b>: <Tag>Auto</Tag> immediate · <Tag>Manual</Tag> staff review ·{' '}
            <Tag>AI</Tag> auto-generated hints.
          </li>
          <li>
            <b>Grading policy</b>: <Tag>Best</Tag> highest score wins · <Tag>Last</Tag> most recent
            attempt counts.
          </li>
        </ul>
      </Card>

      <section id="disallowed" className="scroll-mt-24" />
      <Title level={3}>Disallowed code</Title>
      <Card>
        <Paragraph className="mb-1">
          Add a few <b>specific patterns</b> you don’t want in student code (imports, headers,
          APIs). If a submission contains any listed pattern, it’s <b>flagged</b>.
        </Paragraph>
        <ul className="list-disc pl-5">
          <li>
            <Text code>#include &lt;bits/stdc++.h&gt;</Text>
          </li>
          <li>
            <Text code>import java.net.*</Text>
          </li>
          <li>
            <Text code>Runtime.getRuntime()</Text>
          </li>
        </ul>
        <Paragraph className="mt-3 mb-0">
          Tips: keep patterns precise (avoid broad words), prefer API/package names, and review
          flags to tune the list.
        </Paragraph>
      </Card>

      <section id="tips" className="scroll-mt-24" />
      <Title level={3}>Tips</Title>
      <ul className="list-disc pl-5">
        <li>
          Use <b>Exact</b> for deterministic text. Switch to <b>Percentage</b>/<b>Regex</b> only if
          needed.
        </li>
        <li>
          Keep the <b>delimiter</b> in sync with your Main (e.g., <Text code>###</Text>).
        </li>
        <li>
          <b>Best</b> encourages iteration; <b>Last</b> fits “final submit” workflows.
        </li>
        <li>
          Start with <b>Auto</b> feedback; move to <b>Manual</b> for staggered releases. Enable{' '}
          <b>AI</b> if you want hints.
        </li>
      </ul>

      <section id="json" className="scroll-mt-24" />
      <Title level={3}>Raw config (JSON)</Title>
      <Paragraph className="mb-2">
        The UI manages these values. Field names map to the labels above:
        <Text code className="ml-1">
          marking_scheme
        </Text>
        , <Text code>feedback_scheme</Text>, <Text code>deliminator</Text>,{' '}
        <Text code>grading_policy</Text>, <Text code>limit_attempts</Text>,{' '}
        <Text code>max_attempts</Text>, <Text code>pass_mark</Text>,{' '}
        <Text code>allow_practice_submissions</Text>, <Text code>dissalowed_code</Text>.
      </Paragraph>
      <Card>
        <Paragraph className="mb-2">Defaults:</Paragraph>
        <CodeEditor
          language="json"
          value={DEFAULTS_JSON}
          height={220}
          readOnly
          minimal
          fitContent
          showLineNumbers={false}
          hideCopyButton
        />
        <Paragraph className="mt-4 mb-2">
          Common variant (limited attempts, manual feedback, best score):
        </Paragraph>
        <CodeEditor
          language="json"
          value={COMMON_VARIANT_JSON}
          height={240}
          readOnly
          minimal
          fitContent
          showLineNumbers={false}
          hideCopyButton
        />
      </Card>

      {/* Troubleshooting LAST */}
      <section id="trouble" className="scroll-mt-24" />
      <Title level={3}>Troubleshooting</Title>
      <Collapse
        items={[
          {
            key: 't1',
            label: 'Subsections not showing correctly',
            children: (
              <ul className="list-disc pl-5">
                <li>
                  Ensure your Main prints the same delimiter as configured (e.g.,{' '}
                  <Text code>###</Text>).
                </li>
                <li>
                  Print the subsection label on the same line after the delimiter, once per
                  subsection.
                </li>
              </ul>
            ),
          },
          {
            key: 't2',
            label: 'Students ran out of attempts too soon',
            children: (
              <ul className="list-disc pl-5">
                <li>
                  Disable <b>Limit attempts</b> or increase <b>Max attempts</b>.
                </li>
                <li>
                  Enable <b>Practice submissions</b> so experiments don’t consume the cap.
                </li>
              </ul>
            ),
          },
          {
            key: 't3',
            label: 'Marks differ from expectations',
            children: (
              <ul className="list-disc pl-5">
                <li>
                  Verify the <b>Marking scheme</b> matches how you expect to compare outputs.
                </li>
                <li>
                  Regenerate <b>Memo Output</b> after changing Output/Marking settings.
                </li>
              </ul>
            ),
          },
          {
            key: 't4',
            label: 'Disallowed code not being caught',
            children: (
              <ul className="list-disc pl-5">
                <li>
                  Add precise patterns (e.g., <Text code>{'import java.net.*'}</Text>), then re-run.
                </li>
                <li>Confirm students didn’t obfuscate imports/includes.</li>
              </ul>
            ),
          },
        ]}
      />
    </Space>
  );
}
