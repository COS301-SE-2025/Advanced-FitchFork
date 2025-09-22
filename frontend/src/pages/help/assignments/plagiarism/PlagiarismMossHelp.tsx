import { useEffect, useMemo } from 'react';
import { Typography, Space, Card, Alert, Table, Tag, List } from 'antd';
import { useHelpToc } from '@/context/HelpContext';
import { useBreadcrumbContext } from '@/context/BreadcrumbContext';

const { Title, Paragraph, Text } = Typography;

const toc = [
  { key: 'overview', href: '#overview', title: 'How detection works' },
  { key: 'running-moss', href: '#running-moss', title: 'Running MOSS' },
  { key: 'case-creation', href: '#case-creation', title: 'Automatic cases & fields' },
  { key: 'reviewing-cases', href: '#reviewing-cases', title: 'Reviewing cases in Fitchfork' },
  { key: 'moss-reports', href: '#moss-reports', title: 'Report history & management' },
  { key: 'archives', href: '#archives', title: 'Archive format & offline copies' },
  { key: 'reading-report', href: '#reading-report', title: 'Reading MOSS reports' },
  { key: 'troubleshooting', href: '#troubleshooting', title: 'Troubleshooting' },
];

type StatusRow = {
  key: string;
  label: string;
  meaning: string;
  next: string;
  color: string;
};

const statusColumns = [
  {
    title: 'Status',
    dataIndex: 'label',
    key: 'label',
    width: 140,
    render: (label: string, row: StatusRow) => <Tag color={row.color}>{label}</Tag>,
  },
  {
    title: 'Meaning',
    dataIndex: 'meaning',
    key: 'meaning',
  },
  {
    title: 'Typical next step',
    dataIndex: 'next',
    key: 'next',
  },
];

const statusRows: StatusRow[] = [
  {
    key: 'review',
    label: 'review',
    meaning: 'Fresh cases created by MOSS or by hand. Needs a human pass.',
    next: 'Open the case, inspect matches, then Flag or Mark Reviewed.',
    color: 'blue',
  },
  {
    key: 'flagged',
    label: 'flagged',
    meaning: 'Cases that look suspicious and should be investigated further.',
    next: 'Discuss with teaching team / student before resolving.',
    color: 'orange',
  },
  {
    key: 'reviewed',
    label: 'reviewed',
    meaning: 'Cases that have been acknowledged and resolved.',
    next: 'Leave as-is or delete once documentation is complete.',
    color: 'green',
  },
];

const archiveStructure = [
  {
    title: 'index.html',
    description: 'Landing page with the familiar MOSS summary table and links to each match.',
  },
  {
    title: 'matches/matchX.html',
    description:
      'Side-by-side frames for the Xth match. Each file is rewritten to load locally, including highlighting.',
  },
  {
    title: 'assets/<hash>.{png,svg,css}',
    description:
      'Assets fetched from Stanford and cached with stable hashes so the archive works offline.',
  },
  {
    title: 'archive.zip',
    description: 'Zipped bundle of the entire archive, ready for download from the UI.',
  },
];

const ArchiveStructureList = () => (
  <List
    size="small"
    dataSource={archiveStructure}
    renderItem={(item) => (
      <List.Item>
        <div>
          <Text strong>{item.title}</Text>
          <Paragraph className="!mb-0">{item.description}</Paragraph>
        </div>
      </List.Item>
    )}
  />
);

export default function PlagiarismMossHelp() {
  const { setBreadcrumbLabel } = useBreadcrumbContext();
  const ids = useMemo(() => toc.map((t) => t.href.slice(1)), []);

  useEffect(() => {
    setBreadcrumbLabel('help/assignments/plagiarism/moss', 'Plagiarism & MOSS');
  }, []);

  useHelpToc({
    items: toc,
    ids,
    extra: (
      <Card className="mt-4" size="small" title="At a glance" bordered>
        <ol className="list-decimal pl-5 space-y-1">
          <li>Choose one submission per student (based on the assignment’s grading policy).</li>
          <li>Send those files, plus any base/spec files, to MOSS with your filter settings.</li>
          <li>Store the returned URL as a “MOSS report” with your description and filters.</li>
          <li>
            Parse the report and create plagiarism cases (`status = review`) per matching pair.
          </li>
          <li>
            Mirror the full report to <Text code>moss_archives/&lt;report_id&gt;</Text> and ZIP it.
          </li>
        </ol>
      </Card>
    ),
    onMountScrollToHash: true,
  });

  return (
    <Space direction="vertical" size="small" className="w-full !p-0">
      <Title level={2} className="mb-0">
        Plagiarism &amp; MOSS
      </Title>

      <section id="overview" className="scroll-mt-24" />
      <Title level={3}>How detection works</Title>
      <Paragraph>
        Fitchfork automates the <strong>MOSS (Measure of Software Similarity)</strong> flow for you.
        When you run a job, the backend selects <strong>one submission per student</strong> based on
        the assignment’s grading policy (<Text code>Last</Text> picks the latest eligible attempt,
        <Text code>Best</Text> picks the highest scoring). Any skeleton files you attached as <em>
          Spec
        </em>{' '}
        files are uploaded as base files so MOSS can discount them. The system then:
      </Paragraph>
      <ol className="list-decimal pl-5">
        <li>Sends the curated bundle to Stanford’s MOSS service using your configured language.</li>
        <li>
          Saves the returned URL as a <strong>MOSS report</strong> together with your description
          and filter settings.
        </li>
        <li>
          Parses the HTML results and creates plagiarism cases for each unique submission pair.
        </li>
        <li>
          Launches an archive job that mirrors the entire report (HTML, frames, images) to local
          storage and produces a downloadable ZIP.
        </li>
      </ol>
      <Paragraph>
        The latest URL is also written to <Text code>reports.txt</Text> inside the assignment’s
        storage directory so you always have a quick pointer.
      </Paragraph>
      <Paragraph>
        Behind the scenes, each run is persisted in the <strong>Moss Reports</strong> table with your
        description, filter settings, timestamps, and an archive flag the UI can poll. Parsed matches
        create rows in the <strong>Plagiarism Cases</strong> table, capturing the similarity percent,
        matched line counts, generated description, and a nullable report reference. Those records
        power the Plagiarism Cases list, filters, and graph visualisation.
      </Paragraph>

      <section id="running-moss" className="scroll-mt-24" />
      <Title level={3}>Running MOSS</Title>
      <Paragraph>
        Open an assignment, switch to <strong>Plagiarism</strong>, and use the{' '}
        <strong>Run MOSS</strong>
        button. The modal mirrors the backend rules:
      </Paragraph>
      <ul className="list-disc pl-5">
        <li>
          Provide a short <strong>Report Description</strong>. It is required and becomes the label
          shown in the report list.
        </li>
        <li>
          Choose a file filter: <Text code>All</Text> (compare everything),
          <Text code>Whitelist</Text> (only listed patterns) or <Text code>Blacklist</Text>
          (exclude patterns). Patterns use glob syntax such as <Text code>**/*.cpp</Text>.
        </li>
        <li>
          The assignment’s configured language is sent automatically, so check
          <a href="/help/assignments/config/project" className="ml-1">
            Assignment Config → Language &amp; Mode
          </a>
          before running.
        </li>
        <li>
          Click <strong>Run MOSS</strong>. A background job starts immediately; you will see a toast
          confirming “Started MOSS job”. Refresh the <strong>MOSS Reports</strong> card after a
          minute to track progress.
        </li>
      </ul>
      <Alert
        type="info"
        showIcon
        message="Which submissions are compared?"
        description={
          <div>
            Practice or ignored attempts are skipped automatically. “Last” compares the most recent
            non-practice submission per student; “Best” compares the highest-scoring non-practice
            submission.
          </div>
        }
      />

      <section id="case-creation" className="scroll-mt-24" />
      <Title level={3}>Automatic cases &amp; fields</Title>
      <Paragraph>
        Every match in the report becomes a plagiarism case. Fitchfork deduplicates pairings so each
        student pair appears once per MOSS run. New cases start in <Text code>review</Text> status
        and include:
      </Paragraph>
      <ul className="list-disc pl-5">
        <li>
          <strong>Similarity</strong>: the overall percent reported by MOSS (clamped to 0–100).
        </li>
        <li>
          <strong>Lines matched</strong>: total lines highlighted across all files for the pair.
        </li>
        <li>
          <strong>Description</strong>: a generated summary noting users, similarity band, and
          lines.
        </li>
        <li>
          <strong>Report</strong>: a reference to the MOSS report that produced the case. If you
          delete a report later, the cases remain but the report link is cleared.
        </li>
      </ul>
      <Paragraph>
        You can also add your own cases with <strong>Add Case</strong>. Choose two submissions,
        supply a description, and (optionally) set a similarity percent—useful when you need to
        document academic integrity issues discovered outside MOSS.
      </Paragraph>

      <section id="reviewing-cases" className="scroll-mt-24" />
      <Title level={3}>Reviewing cases in Fitchfork</Title>
      <Paragraph>
        The cases list supports grid and table views. Filter by status, search by username, or focus
        on cases from a specific MOSS run using the <strong>Report</strong> filter (Fitchfork shows
        your report descriptions there). Columns for <strong>Similarity</strong> and{' '}
        <strong>Lines</strong>
        are sortable so you can triage the highest-risk pairs first.
      </Paragraph>
      <Paragraph>Case actions align with the status lifecycle:</Paragraph>
      <Table
        size="small"
        columns={statusColumns}
        dataSource={statusRows}
        pagination={false}
        className="max-w-4xl"
      />
      <Paragraph>
        Use <strong>Flag</strong> to escalate, then <strong>Mark Reviewed</strong> once you have
        documented the outcome. Bulk delete is available for clearing test runs. For a birds-eye
        view, open <strong>View Graph</strong>; the 2D/3D graph groups users as nodes, colours edges
        by similarity, and lets you filter by status, percent range, username, or report.
      </Paragraph>

      <section id="moss-reports" className="scroll-mt-24" />
      <Title level={3}>Report history &amp; management</Title>
      <Paragraph>
        The <strong>MOSS Reports</strong> card records every run. Each entry surfaces the same key
        fields stored in the Moss Reports table, so you can see at a glance:
      </Paragraph>
      <ul className="list-disc pl-5">
        <li>The original MOSS URL and timestamp.</li>
        <li>Your description (shown in tooltips and filters).</li>
        <li>Filter mode and glob patterns used for the run.</li>
        <li>
          Whether a local archive is available (<Text code>has_archive</Text>).
        </li>
      </ul>
      <Paragraph>
        Click <strong>Open</strong> to jump to the live MOSS site, or use the actions menu to view
        details, download the ZIP (once the archive job finishes), or delete the report. Deleting a
        report also removes its local archive folder; existing cases remain but their report
        reference is cleared so you never lose review notes.
      </Paragraph>
      <Alert
        showIcon
        type="success"
        message="Latest ZIP quick action"
        description="The button beside Run MOSS always downloads the newest archived ZIP if it is ready."
      />

      <section id="archives" className="scroll-mt-24" />
      <Title level={3}>Archive format &amp; offline copies</Title>
      <Paragraph>
        Archives are stored alongside each assignment in a <Text code>moss_archives/&lt;report_id&gt;</Text>
        folder on the server. The folder name matches the report ID, so removing a report removes its
        archive directory and ZIP. When the background job finishes you will find:
      </Paragraph>
      <ArchiveStructureList />
      <Paragraph>
        The HTML is rewritten so <Text code>index.html</Text> works offline—open it in any browser
        to explore matches even after the Stanford URL expires. Fitchfork also drops a pointer at
        <Text code>moss_archives/&lt;report_id&gt;/archive.zip</Text>
        and exposes that ZIP through the UI download buttons.
      </Paragraph>

      <section id="reading-report" className="scroll-mt-24" />
      <Title level={3}>Reading MOSS reports</Title>
      <Paragraph>
        Start with <strong>index.html</strong>. The table shows each pair, their overall similarity,
        and the number of matching lines. Click a pair to open <Text code>matchX.html</Text>, which
        displays the two submissions side-by-side with coloured regions for each match block.
        Hovering over a block highlights the corresponding code in both panes and lists the line
        ranges.
      </Paragraph>
      <Paragraph>Tips while reviewing:</Paragraph>
      <ul className="list-disc pl-5">
        <li>Use the line ranges to locate the same code in Fitchfork’s submission viewer.</li>
        <li>
          Large matches across multiple files often indicate shared projects—check for identical
          filenames and directory structures.
        </li>
        <li>
          The <strong>Lines Matched</strong> column in Fitchfork comes from the same totals you see
          in the match pages.
        </li>
      </ul>

      <section id="troubleshooting" className="scroll-mt-24" />
      <Title level={3}>Troubleshooting</Title>
      <ul className="list-disc pl-5">
        <li>
          <strong>ZIP button disabled?</strong> The archive job is still running. Refresh the
          reports list; once <Text code>has_archive</Text> flips to true the button becomes active.
        </li>
        <li>
          <strong>Patterns rejected?</strong> Whitelist/Blacklist require at least one glob pattern;
          All must have none. Match what you enter in the Run MOSS modal.
        </li>
        <li>
          <strong>No cases after a run?</strong> Ensure the submissions you expect are not marked as
          practice or ignored, and confirm the language in Assignment Config matches the student
          code.
        </li>
        <li>
          <strong>Need to re-run for the same cohort?</strong> Delete the previous report (to clear
          the archive) or leave it in place—new runs simply add another report and new cases.
        </li>
      </ul>
    </Space>
  );
}
