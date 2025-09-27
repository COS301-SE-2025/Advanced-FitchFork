// src/pages/help/assignments/MarkAllocatorHelp.tsx
import { useEffect, useMemo } from 'react';
import { Typography, Card, Alert, Space, Collapse, Table, Descriptions, Tag } from 'antd';
import { useHelpToc } from '@/context/HelpContext';
import { useBreadcrumbContext } from '@/context/BreadcrumbContext';
import { useViewSlot } from '@/context/ViewSlotContext';

const { Title, Paragraph, Text } = Typography;

const toc = [
  { key: 'what', href: '#what', title: 'What is Mark Allocation?' },
  { key: 'where', href: '#where', title: 'Where to find it' },
  { key: 'how', href: '#how', title: 'How it works' },
  { key: 'edit', href: '#edit', title: 'Edit allocations' },
  { key: 'labels', href: '#labels', title: 'Labels & subsections' },
  { key: 'tips', href: '#tips', title: 'Tips' },
  { key: 'trouble', href: '#trouble', title: 'Troubleshooting' }, // keep last
];

const sampleCols = [
  { title: 'Subsection label', dataIndex: 'label', key: 'label', width: 260 },
  { title: 'Marks', dataIndex: 'marks', key: 'marks', width: 120 },
  { title: 'Notes', dataIndex: 'notes', key: 'notes' },
];

const sampleRows = [
  {
    key: 's1',
    label: 'Task1Subtask1',
    marks: 3,
    notes: 'Lines printed under this label in Memo Output are compared for this slice.',
  },
  {
    key: 's2',
    label: 'Task1Subtask2',
    marks: 3,
    notes: 'Same idea; label comes from your delimiter print in Main.',
  },
  {
    key: 's3',
    label: 'Task1Subtask3',
    marks: 3,
    notes: 'Task total = sum of all subsection marks.',
  },
];

export default function MarkAllocatorHelp() {
  const { setBreadcrumbLabel } = useBreadcrumbContext();
  const ids = useMemo(() => toc.map((t) => t.href.slice(1)), []);
  const { setValue, setBackTo } = useViewSlot();

  useEffect(() => {
    setValue(
      <Typography.Text className="text-base font-medium text-gray-900 dark:text-gray-100 truncate">
        Mark Allocator
      </Typography.Text>,
    );
    setBackTo('/help');
  }, []);

  useEffect(() => {
    setBreadcrumbLabel('help/assignments/mark-allocator', 'Mark Allocation');
  }, []);

  useHelpToc({
    items: toc,
    ids,
    extra: (
      <Card className="mt-4" size="small" title="Quick facts" bordered>
        <ul className="list-disc pl-5">
          <li>
            Allocate <b>marks</b> per <b>task</b> and per <b>subsection label</b>.
          </li>
          <li>
            Open <b>Tasks → pick a task → pick a subsection</b> to adjust its marks.
          </li>
          <li>Task total = sum of subsection marks. Assignment total = sum of task totals.</li>
        </ul>
      </Card>
    ),
    onMountScrollToHash: true,
  });

  return (
    <Space direction="vertical" size="small" className="w-full !p-0">
      <Title level={2} className="mb-0">
        Mark Allocation
      </Title>

      <section id="what" className="scroll-mt-24" />
      <Title level={3}>What is Mark Allocation?</Title>
      <Paragraph className="mb-0">
        Mark Allocation lets you decide how many <b>marks</b> each <b>task</b> and each
        <b> subsection</b> (label) is worth. These weights work together with your
        <a href="/help/assignments/memo-output" className="ml-1">
          Memo Output
        </a>{' '}
        and
        <a href="/help/assignments/config/marking" className="ml-1">
          Marking
        </a>{' '}
        scheme to produce scores.
      </Paragraph>

      <section id="where" className="scroll-mt-24" />
      <Title level={3}>Where to find it</Title>
      <ol className="list-decimal pl-5">
        <li>
          Open the assignment’s <b>Tasks</b> page.
        </li>
        <li>
          Select the <b>specific task</b> you want to configure.
        </li>
        <li>
          Choose a <b>Subsection</b> within that task (the label printed by your delimiter).
        </li>
        <li>
          Edit the <b>Marks</b> for that subsection. Repeat for other subsections in the task.
        </li>
      </ol>

      <section id="how" className="scroll-mt-24" />
      <Title level={3}>How it works</Title>
      <Descriptions bordered size="middle" column={1} className="mt-2">
        <Descriptions.Item label="Scope">
          Allocation happens at the <Tag color="blue">task</Tag> level, split across its{' '}
          <Tag color="geekblue">subsections</Tag>.
        </Descriptions.Item>
        <Descriptions.Item label="Totals">
          <b>Task total</b> = sum of its subsection <b>marks</b>. <b>Assignment total</b> = sum of
          all task totals.
        </Descriptions.Item>
        <Descriptions.Item label="What gets marks">
          A subsection earns <b>marks</b> when the student’s output for that label satisfies your
          <a href="/help/assignments/config/marking" className="ml-1">
            Marking
          </a>{' '}
          scheme.
        </Descriptions.Item>
      </Descriptions>

      <section id="example" className="scroll-mt-24" />
      <Card className="mt-3" bordered>
        <Paragraph className="mb-2">Example allocation for one task:</Paragraph>

        {/* Desktop table */}
        <div className="hidden md:block">
          <Table
            size="small"
            columns={sampleCols}
            dataSource={sampleRows}
            pagination={false}
            scroll={{ x: true }}
          />
        </div>

        {/* Mobile cards */}
        <div className="block md:hidden !space-y-3">
          {sampleRows.map((r) => (
            <Card
              key={r.key}
              size="small"
              title={<div className="font-semibold">{r.label}</div>}
              extra={
                <Tag color="purple">
                  {r.marks} mark{r.marks === 1 ? '' : 's'}
                </Tag>
              }
              bordered
            >
              <div className="text-xs uppercase tracking-wide text-gray-500 dark:text-gray-400 mb-1">
                Notes
              </div>
              <div className="text-sm">{r.notes}</div>
            </Card>
          ))}
        </div>

        <Paragraph className="mt-2 mb-0">
          In this example, the task total is <b>9 marks</b> (3 + 3 + 3).
        </Paragraph>
      </Card>

      <section id="edit" className="scroll-mt-24" />
      <Title level={3}>Edit allocations</Title>
      <ul className="list-disc pl-5">
        <li>
          Open <b>Tasks → Task</b>, pick a <b>Subsection</b>, enter the <b>Marks</b>, and save.
        </li>
        <li>
          Repeat for all subsections you want to weigh. You can make some 0 if they shouldn’t count.
        </li>
        <li>
          Changes apply to future runs immediately; existing results reflect the new weights when
          viewed.
        </li>
      </ul>
      <Alert
        className="mt-2"
        type="info"
        showIcon
        message="Labels drive the buckets"
        description={
          <>
            Subsections come from the labels you print in <b>Main</b> using the delimiter (
            <Text code>###</Text>). If you rename a label, it appears as a new subsection here.
            Keep labels stable unless you intend to reallocate.
          </>
        }
      />

      <section id="labels" className="scroll-mt-24" />
      <Title level={3}>Labels & subsections</Title>
      <Paragraph className="mb-0">
        In your program’s <b>Main</b>, print <Text code>###</Text> followed by the subsection name
        to start a labeled block. Those names are what you see under each Task when assigning{' '}
        <b>marks</b>. See <a href="/help/assignments/memo-output">Memo Output</a> for examples.
      </Paragraph>

      <section id="tips" className="scroll-mt-24" />
      <Title level={3}>Tips</Title>
      <ul className="list-disc pl-5">
        <li>
          Decide a consistent <b>granularity</b> (few big subsections vs. many small ones) to match
          your rubric.
        </li>
        <li>
          Keep <b>labels stable</b> across updates to avoid accidental new buckets.
        </li>
        <li>
          If your assignment also uses <a href="/help/assignments/code-coverage">Code Coverage</a>,
          plan weights so coverage marks (if any) fit naturally with subsection marks.
        </li>
      </ul>

      {/* Troubleshooting LAST */}
      <section id="trouble" className="scroll-mt-24" />
      <Title level={3}>Troubleshooting</Title>
      <Collapse
        items={[
          {
            key: 't1',
            label: '“I don’t see any subsections to allocate”',
            children: (
              <ul className="list-disc pl-5">
                <li>
                  Make sure your <b>Main</b> prints labels using <Text code>###</Text> before the
                  lines in that section.
                </li>
                <li>
                  Regenerate <a href="/help/assignments/memo-output">Memo Output</a> so the system
                  learns the labels.
                </li>
              </ul>
            ),
          },
          {
            key: 't2',
            label: '“My totals don’t add up”',
            children: (
              <ul className="list-disc pl-5">
                <li>
                  Check each subsection’s <b>marks</b> under the task; the task total is their sum.
                </li>
                <li>Assignment total updates automatically from all task totals.</li>
              </ul>
            ),
          },
          {
            key: 't3',
            label: '“A label disappeared or a new one appeared”',
            children: (
              <Paragraph>
                You likely changed the printed label in <b>Main</b>. Keep label names stable, or
                adjust the new subsection’s marks and (if needed) regenerate Memo Output.
              </Paragraph>
            ),
          },
          {
            key: 't4',
            label: '“Edits saved, but I still see old behaviour”',
            children: (
              <Paragraph>
                Allocation changes take effect immediately, but if you also changed outputs/labels,
                regenerate <b>Memo Output</b> so comparisons align.
              </Paragraph>
            ),
          },
        ]}
      />
    </Space>
  );
}
