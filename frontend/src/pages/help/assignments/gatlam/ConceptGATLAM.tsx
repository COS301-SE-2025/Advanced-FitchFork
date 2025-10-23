// src/pages/help/assignments/gatlam/ConceptGATLAM.tsx
import { useEffect, useMemo } from 'react';
import { Typography, Card, Space, Collapse, Table, Alert, Tag } from 'antd';
import { useHelpToc } from '@/context/HelpContext';
import { CodeEditor, GatlamLink } from '@/components/common';
import { useBreadcrumbContext } from '@/context/BreadcrumbContext';
import { useViewSlot } from '@/context/ViewSlotContext';

const { Title, Paragraph, Text } = Typography;

const toc = [
  { key: 'what', href: '#what', title: 'What is GATLAM?' },
  { key: 'when', href: '#when', title: 'When to use it' },
  { key: 'pipeline', href: '#pipeline', title: 'Interpreter pipeline' },
  { key: 'eval', href: '#eval', title: 'What the interpreter checks' },
  { key: 'labels', href: '#labels', title: 'Labeled output (delimiter)' },
  { key: 'ga', href: '#ga', title: 'How the GA evolves candidates' },
  { key: 'ops', href: '#ops', title: 'Operators (crossover & mutation)' },
  { key: 'params', href: '#params', title: 'Key knobs (conceptually)' },
  { key: 'examples', href: '#examples', title: 'Examples' },
  { key: 'tips', href: '#tips', title: 'Tips' },
  { key: 'links', href: '#links', title: 'See also' },
  { key: 'trouble', href: '#trouble', title: 'Troubleshooting' }, // keep last
];

// --- small samples for the viewer ---
const SAMPLE_STDOUT = `### Task1:Step1
12
34
### Task1:Step2
OK
RUNTIME_MS: 250
Retcode: 0
`;

const SAMPLE_MEMO = `### Task1:Step1
12
34
### Task1:Step2
OK
`;

const checksCols = [
  { title: 'Check', dataIndex: 'check', key: 'check', width: 220 },
  { title: 'Looks at', dataIndex: 'source', key: 'source', width: 160 },
  { title: 'Passes when…', dataIndex: 'meaning', key: 'meaning' },
];

const checksRows = [
  {
    key: 'safety',
    check: 'Safety',
    source: 'stderr',
    meaning: 'No low-level crash/ASan-like patterns (e.g., use-after-free, invalid pointer).',
  },
  {
    key: 'term',
    check: 'Proper termination',
    source: 'retcode',
    meaning: 'Exit code is in the allowed list (default: 0).',
  },
  {
    key: 'segv',
    check: 'Segmentation fault',
    source: 'stderr',
    meaning: 'No segfault markers (C++/Java patterns).',
  },
  {
    key: 'ex',
    check: 'Exceptions',
    source: 'stderr',
    meaning: 'No uncaught exceptions / fatal JVM errors.',
  },
  {
    key: 'time',
    check: 'Execution time bound',
    source: 'RUNTIME_MS',
    meaning: 'If a bound is set, reported runtime (ms) ≤ bound.',
  },
  {
    key: 'illegal',
    check: 'Illegal output',
    source: 'stdout',
    meaning: 'No line exactly equals a forbidden output entry (trimmed).',
  },
  {
    key: 'exact',
    check: 'Expected (Exact)',
    source: 'stdout + labels',
    meaning: 'Within each labeled section, lines exactly match the Memo lines.',
  },
  {
    key: 'contains',
    check: 'Expected (Contains)',
    source: 'stdout + labels',
    meaning: 'Within each label, Memo lines appear as substrings of output lines.',
  },
];

const opsCols = [
  { title: 'Operator', dataIndex: 'name', key: 'name', width: 220 },
  { title: 'What it does', dataIndex: 'desc', key: 'desc' },
  { title: 'Notes', dataIndex: 'notes', key: 'notes', width: 260 },
];

const opsRows = [
  {
    key: 'x1',
    name: 'Crossover: One-point',
    desc: 'Child takes prefix from Parent A and suffix from Parent B at a random cut.',
    notes: 'Good baseline; preserves contiguous gene segments.',
  },
  {
    key: 'x2',
    name: 'Crossover: Two-point',
    desc: 'Child takes a middle segment from Parent B, rest from Parent A.',
    notes: 'Mixes a block while keeping edges intact.',
  },
  {
    key: 'xu',
    name: 'Crossover: Uniform',
    desc: 'Each bit is chosen independently from Parent A or B.',
    notes: 'High mixing; can disrupt building blocks.',
  },
  {
    key: 'mflip',
    name: 'Mutation: Bit flip',
    desc: 'Each bit flips with some probability.',
    notes: 'Classic mutator for bitstring genes.',
  },
  {
    key: 'mswap',
    name: 'Mutation: Swap',
    desc: 'Randomly swaps two bit positions.',
    notes: 'Maintains bit counts; mild shuffle.',
  },
  {
    key: 'mscr',
    name: 'Mutation: Scramble',
    desc: 'Randomly shuffles a contiguous bit segment.',
    notes: 'Bigger local reordering; preserves multiset.',
  },
];

const knobsCols = [
  { title: 'Conceptual knob', dataIndex: 'k', key: 'k', width: 240 },
  { title: 'What it controls', dataIndex: 'w', key: 'w' },
  { title: 'Typical default', dataIndex: 'd', key: 'd', width: 180 },
];

const knobsRows = [
  { key: 'pop', k: 'Population size', w: 'How many candidates per generation.', d: '100' },
  { key: 'gens', k: 'Generations', w: 'How long the GA runs (evolution depth).', d: '50' },
  { key: 'sel', k: 'Selection size', w: 'How many candidates compete/seed reproduction.', d: '20' },
  {
    key: 'repr',
    k: 'Reproduction probability',
    w: 'How often we perform crossover vs cloning.',
    d: '0.8',
  },
  {
    key: 'cx',
    k: 'Crossover probability',
    w: 'Strength/usage of crossover (variant-dependent).',
    d: '0.9',
  },
  { key: 'mut', k: 'Mutation probability', w: 'Chance of mutation on a child.', d: '0.01' },
  {
    key: 'genes',
    k: 'Genes (min/max per gene)',
    w: 'Range of each integer gene encoded as bits.',
    d: 'e.g., −5..5; −4..9',
  },
  { key: 'cxType', k: 'Crossover type', w: 'One-point / Two-point / Uniform.', d: 'One-point' },
  { key: 'mutType', k: 'Mutation type', w: 'Bit-flip / Swap / Scramble.', d: 'Bit-flip' },
  {
    key: 'para',
    k: 'Parallel chromosomes',
    w: 'How many candidates can be evaluated at once.',
    d: '4',
  },
];

export default function ConceptGATLAM() {
  const { setBreadcrumbLabel } = useBreadcrumbContext();
  const ids = useMemo(() => toc.map((t) => t.href.slice(1)), []);
  const { setValue, setBackTo } = useViewSlot();

  useEffect(() => {
    setValue(
      <Typography.Text className="text-base font-medium text-gray-900 dark:text-gray-100 truncate">
        GATLAM & Interpreter
      </Typography.Text>,
    );
    setBackTo('/help');
  }, []);

  useEffect(() => {
    setBreadcrumbLabel('help/assignments/gatlam', 'GATLAM & Interpreter');
  }, []);

  useHelpToc({
    items: toc,
    ids,
    extra: (
      <Card className="mt-4" size="small" title="Quick facts" bordered>
        <ul className="list-disc pl-5">
          <li>
            <GatlamLink tone="inherit" icon={false} underline={false}>
              GATLAM
            </GatlamLink>{' '}
            = Genetic Algorithm + task interpreter.
          </li>
          <li>
            Interpreter checks safety, termination, time, exceptions, illegal output, and labeled
            matches.
          </li>
          <li>
            Use the <Text code>###</Text> delimiter to label subsections in stdout.
          </li>
          <li>
            Outputs must include <Text code>Retcode:</Text> and (optionally){' '}
            <Text code>RUNTIME_MS:</Text> markers.
          </li>
        </ul>
      </Card>
    ),
    onMountScrollToHash: true,
  });

  return (
    <Space direction="vertical" size="small" className="w-full !p-0">
      <Title level={2} className="mb-0">
        GATLAM &amp; Interpreter
      </Title>

      <section id="what" className="scroll-mt-24" />
      <Title level={3}>What is GATLAM?</Title>
      <Paragraph className="mb-0">
        <GatlamLink tone="inherit" icon={false} underline={false}>
          GATLAM
        </GatlamLink>{' '}
        lets the grader explore a space of candidate inputs or parameters using a
        <b> genetic algorithm (GA)</b>. Each candidate is run through the <b>interpreter</b>, which
        parses program output and checks a set of properties. The results are turned into a fitness
        signal that guides the GA to better candidates over several generations.
      </Paragraph>

      <section id="when" className="scroll-mt-24" />
      <Title level={3}>When to use it</Title>
      <ul className="list-disc pl-5">
        <li>
          To automatically search inputs that trigger crashes, exceptions, or time limit violations.
        </li>
        <li>To find edge cases where student output deviates from the memo.</li>
        <li>To optimize simple numeric parameters or seeds for stress tests.</li>
      </ul>

      <section id="pipeline" className="scroll-mt-24" />
      <Title level={3}>Interpreter pipeline</Title>
      <ol className="list-decimal pl-5">
        <li>
          <b>Run candidate</b> (under your normal Execution/Output limits).
        </li>
        <li>
          <b>Parse</b> the blob into <Tag>stdout</Tag>, <Tag>stderr</Tag>, <Tag>Retcode</Tag>, and
          optional <Tag>RUNTIME_MS</Tag>.
        </li>
        <li>
          <b>Evaluate checks</b> (safety, termination, exceptions, labeled matches, etc.).
        </li>
        <li>
          <b>Aggregate</b> into two metrics: <Text code>ltl_milli</Text> (fraction of property
          violations) and <Text code>fail_milli</Text> (fraction of failed tasks).
        </li>
        <li>
          <b>Report fitness</b> to the GA driver, which selects, crosses, and mutates for the next
          generation.
        </li>
      </ol>

      <section id="eval" className="scroll-mt-24" />
      <Title level={3}>What the interpreter checks</Title>

      {/* Desktop table */}
      <div className="hidden md:block">
        <Table
          className="mt-2"
          size="small"
          columns={checksCols}
          dataSource={checksRows}
          pagination={false}
          scroll={{ x: true }}
        />
      </div>
      {/* Mobile cards */}
      <div className="block md:hidden mt-2 !space-y-3">
        {checksRows.map((r) => (
          <Card key={r.key} size="small" title={<div className="font-semibold">{r.check}</div>}>
            <div className="text-xs uppercase tracking-wide text-gray-500 dark:text-gray-400 mb-1">
              Looks at
            </div>
            <div className="text-sm mb-2">{r.source}</div>
            <div className="text-xs uppercase tracking-wide text-gray-500 dark:text-gray-400 mb-1">
              Passes when…
            </div>
            <div className="text-sm">{r.meaning}</div>
          </Card>
        ))}
      </div>

      <Alert
        className="mt-3"
        type="info"
        showIcon
        message="Two scores are produced"
        description={
          <>
            <b>ltl_milli</b>: proportion of property violations across all checks (0–1000).&nbsp;
            <b>fail_milli</b>: proportion of tasks considered failed (0–1000). The GA’s fitness
            function penalizes larger values.
          </>
        }
      />

      <section id="labels" className="scroll-mt-24" />
      <Title level={3}>Labeled output (delimiter)</Title>
      <Paragraph className="mb-1">
        Use the delimiter (default <Text code>###</Text>) to name subsections in <b>stdout</b>.
        The interpreter compares the lines inside each label to the Memo’s lines for the same label
        (both exact and contains).
      </Paragraph>
      <Card>
        <Space direction="vertical" size="small" className="w-full">
          <CodeEditor
            title="Student run (stdout)"
            language="plaintext"
            value={SAMPLE_STDOUT}
            height={180}
            readOnly
            minimal
            fitContent
            showLineNumbers={false}
            hideCopyButton
          />
          <CodeEditor
            title="Memo output (reference)"
            language="plaintext"
            value={SAMPLE_MEMO}
            height={140}
            readOnly
            minimal
            fitContent
            showLineNumbers={false}
            hideCopyButton
          />
        </Space>
      </Card>
      <Paragraph className="mt-2 mb-0">
        Markers the parser understands (case-insensitive): <Text code>Retcode:</Text>,{' '}
        <Text code>RUNTIME_MS:</Text>, and an optional <Text code>STDERR:</Text> section header.
      </Paragraph>

      <section id="ga" className="scroll-mt-24" />
      <Title level={3}>How the GA evolves candidates</Title>
      <Paragraph className="mb-0">
        Each generation, the GA <b>selects</b> candidates (roulette-style by fitness), performs
        <b> crossover</b> with some probability, then applies <b>mutation</b>. Candidates are
        bit-strings representing integer genes (sign + magnitude). Gene ranges come from your
        assignment’s GATLAM settings.
      </Paragraph>

      <section id="ops" className="scroll-mt-24" />
      <Title level={3}>Operators (crossover &amp; mutation)</Title>

      {/* Desktop table */}
      <div className="hidden md:block">
        <Table
          className="mt-2"
          size="small"
          columns={opsCols}
          dataSource={opsRows}
          pagination={false}
          scroll={{ x: true }}
        />
      </div>
      {/* Mobile cards */}
      <div className="block md:hidden mt-2 !space-y-3">
        {opsRows.map((r) => (
          <Card key={r.key} size="small" title={<div className="font-semibold">{r.name}</div>}>
            <div className="text-xs uppercase tracking-wide text-gray-500 dark:text-gray-400 mb-1">
              What it does
            </div>
            <div className="text-sm mb-2">{r.desc}</div>
            <div className="text-xs uppercase tracking-wide text-gray-500 dark:text-gray-400 mb-1">
              Notes
            </div>
            <div className="text-sm">{r.notes}</div>
          </Card>
        ))}
      </div>

      <section id="params" className="scroll-mt-24" />
      <Title level={3}>Key knobs (conceptually)</Title>

      {/* Desktop table */}
      <div className="hidden md:block">
        <Table
          className="mt-2"
          size="small"
          columns={knobsCols}
          dataSource={knobsRows}
          pagination={false}
          scroll={{ x: true }}
        />
      </div>
      {/* Mobile cards */}
      <div className="block md:hidden mt-2 !space-y-3">
        {knobsRows.map((r) => (
          <Card key={r.key} size="small" title={<div className="font-semibold">{r.k}</div>}>
            <div className="text-xs uppercase tracking-wide text-gray-500 dark:text-gray-400 mb-1">
              What it controls
            </div>
            <div className="text-sm mb-2">{r.w}</div>
            <div className="text-xs uppercase tracking-wide text-gray-500 dark:text-gray-400 mb-1">
              Typical default
            </div>
            <div className="text-sm">{r.d}</div>
          </Card>
        ))}
      </div>

      <Paragraph className="mt-2 mb-0">
        Gene bit-width is picked from the <b>largest absolute</b> min/max across all genes, then
        each gene value is encoded using sign+magnitude to that width.
      </Paragraph>

      <section id="examples" className="scroll-mt-24" />
      <Title level={3}>Examples</Title>
      <ul className="list-disc pl-5">
        <li>
          <b>Crash hunting:</b> Penalize safety/segfault/exception violations heavily, GA searches
          inputs that trigger them.
        </li>
        <li>
          <b>Spec conformance:</b> Use labels; GA searches seeds that produce memo-divergent lines.
        </li>
        <li>
          <b>Runtime bounding:</b> Set <Text code>RUNTIME_MS:</Text> and a limit; GA discovers slow
          paths (violations).
        </li>
      </ul>

      <section id="tips" className="scroll-mt-24" />
      <Title level={3}>Tips</Title>
      <ul className="list-disc pl-5">
        <li>Keep outputs deterministic and concise; noisy logs make matching harder.</li>
        <li>
          Always print <Text code>Retcode:</Text> (and <Text code>RUNTIME_MS:</Text> if timing
          matters).
        </li>
        <li>
          Use meaningful labels (e.g., <Text code>Task1:Parse</Text>, <Text code>Task1:Eval</Text>)
          so diffs are obvious.
        </li>
        <li>Start with modest population/generations, then scale if search plateaus.</li>
      </ul>

      <section id="links" className="scroll-mt-24" />
      <Title level={3}>See also</Title>
      <ul className="list-disc pl-5">
        <li>
          Configure GA parameters in{' '}
          <a href="/help/assignments/config/gatlam">Assignment Config → GATLAM</a>.
        </li>
        <li>Ensure the values you care about are printed to stdout; that stream is always saved.</li>
        <li>
          Delimiter and memo labeling: <a href="/help/assignments/files/main-files">Main File</a>{' '}
          and <a href="/help/assignments/memo-output">Memo Output</a>.
        </li>
      </ul>

      {/* Troubleshooting LAST */}
      <section id="trouble" className="scroll-mt-24" />
      <Title level={3}>Troubleshooting</Title>
      <Collapse
        items={[
          {
            key: 't1',
            label: '“No labels found in output”',
            children: (
              <ul className="list-disc pl-5">
                <li>
                  Print the delimiter (default <Text code>###</Text>) before each subsection name.
                </li>
                <li>
                  Make sure you write labels to <b>stdout</b>, not stderr.
                </li>
              </ul>
            ),
          },
          {
            key: 't2',
            label: 'Termination check always fails',
            children: (
              <ul className="list-disc pl-5">
                <li>
                  Print <Text code>Retcode: &lt;n&gt;</Text> in the output, or ensure your runner
                  records it.
                </li>
                <li>Confirm the allowed exit codes include the one you intend (default is 0).</li>
              </ul>
            ),
          },
          {
            key: 't3',
            label: 'Runtime bound never triggers',
            children: (
              <ul className="list-disc pl-5">
                <li>
                  Print <Text code>RUNTIME_MS: &lt;n&gt;</Text> and set a max runtime in GATLAM Task
                  Spec.
                </li>
              </ul>
            ),
          },
          {
            key: 't4',
            label: 'False “Illegal output” hits',
            children: (
              <ul className="list-disc pl-5">
                <li>
                  Remember: the property uses <b>exact trimmed line</b> matching for forbidden
                  items.
                </li>
                <li>Remove overly broad entries; prefer specific lines over substrings.</li>
              </ul>
            ),
          },
          {
            key: 't5',
            label: 'Crashes not being detected (non C++/Java)',
            children: (
              <Paragraph className="mb-0">
                Crash/exception patterns are language-aware for C++ and Java. Other languages
                default to “safe”.
              </Paragraph>
            ),
          },
        ]}
      />
    </Space>
  );
}
