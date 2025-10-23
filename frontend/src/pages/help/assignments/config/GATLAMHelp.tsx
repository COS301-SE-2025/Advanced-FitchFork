// src/pages/help/assignments/config/GATLAMHelp.tsx
import { useEffect, useMemo } from 'react';
import { Typography, Card, Space, Collapse, Table, Alert } from 'antd';
import { useHelpToc } from '@/context/HelpContext';
import { CodeEditor, GatlamLink } from '@/components/common';
import { useBreadcrumbContext } from '@/context/BreadcrumbContext';
import { useViewSlot } from '@/context/ViewSlotContext';

const { Title, Paragraph, Text } = Typography;

const toc = [
  { key: 'what', href: '#what', title: 'What is GATLAM mode?' },
  { key: 'options', href: '#options', title: 'Options & defaults' },
  { key: 'tips', href: '#tips', title: 'Tips' },
  { key: 'json', href: '#json', title: 'Raw config (JSON)' },
  { key: 'trouble', href: '#trouble', title: 'Troubleshooting' }, // keep last
];

const DEFAULTS_JSON = `{
  "gatlam": {
    "population_size": 100,
    "number_of_generations": 50,
    "selection_size": 20,
    "reproduction_probability": 0.8,
    "crossover_probability": 0.9,
    "mutation_probability": 0.01,
    "genes": [
      { "min_value": -5, "max_value": 5 },
      { "min_value": -4, "max_value": 9 }
    ],
    "crossover_type": "onepoint",
    "mutation_type": "bitflip",
    "omega1": 0.5,
    "omega2": 0.3,
    "omega3": 0.2,
    "task_spec": {
      "valid_return_codes": [0],
      "max_runtime_ms": null,
      "forbidden_outputs": []
    },
    "max_parallel_chromosomes": 4,
    "verbose": false
  }
}`;

const EXAMPLE_JSON = `{
  "gatlam": {
    "population_size": 150,
    "number_of_generations": 60,
    "selection_size": 30,
    "reproduction_probability": 0.85,
    "crossover_probability": 0.9,
    "mutation_probability": 0.02,
    "genes": [
      { "min_value": -10, "max_value": 10 },
      { "min_value": 0, "max_value": 20 }
    ],
    "crossover_type": "uniform",
    "mutation_type": "scramble",
    "omega1": 0.5,
    "omega2": 0.3,
    "omega3": 0.2,
    "task_spec": {
      "valid_return_codes": [0],
      "max_runtime_ms": 2000,
      "forbidden_outputs": ["forbidden", "BAD"]
    },
    "max_parallel_chromosomes": 6,
    "verbose": false
  }
}`;

// Human-friendly table (short and sweet)
const optionCols = [
  { title: 'Setting', dataIndex: 'setting', key: 'setting', width: 260 },
  { title: 'What it does', dataIndex: 'meaning', key: 'meaning' },
  { title: 'Options / Format', dataIndex: 'options', key: 'options', width: 280 },
  { title: 'Default', dataIndex: 'def', key: 'def', width: 140 },
];

const optionRows = [
  {
    key: 'pop',
    setting: 'Population size',
    meaning: 'Chromosomes per generation.',
    options: 'Integer ≥ 2',
    def: '100',
  },
  {
    key: 'gens',
    setting: 'Generations',
    meaning: 'How many evolve steps to run.',
    options: 'Integer ≥ 1',
    def: '50',
  },
  {
    key: 'select',
    setting: 'Selection size',
    meaning: 'How many candidates survive/participate in selection.',
    options: 'Integer (≤ population)',
    def: '20',
  },
  {
    key: 'repr',
    setting: 'Reproduction probability',
    meaning: 'Chance to perform crossover when producing a child.',
    options: '0.0 – 1.0',
    def: '0.8',
  },
  {
    key: 'xover',
    setting: 'Crossover type',
    meaning: 'How parents are combined.',
    options: <span>One-point / Two-point / Uniform</span>,
    def: 'One-point',
  },
  {
    key: 'mutp',
    setting: 'Mutation probability',
    meaning: 'Per-child chance to mutate (operator below).',
    options: '0.0 – 1.0',
    def: '0.01',
  },
  {
    key: 'mutop',
    setting: 'Mutation operator',
    meaning: 'How children are mutated.',
    options: 'Bit-flip / Swap / Scramble',
    def: 'Bit-flip',
  },
  {
    key: 'genes',
    setting: 'Genes (search ranges)',
    meaning: 'Bounds for each tunable gene.',
    options: 'List of {min_value, max_value}',
    def: '[{-5..5}, {-4..9}]',
  },
  {
    key: 'task_ret',
    setting: 'Valid return codes',
    meaning: 'What exit codes count as “proper termination”.',
    options: 'List of integers (e.g., [0])',
    def: '[0]',
  },
  {
    key: 'task_time',
    setting: 'Time bound (ms)',
    meaning: 'Flags if runtime exceeds this per task.',
    options: 'Integer ms or empty',
    def: '—',
  },
  {
    key: 'task_forb',
    setting: 'Forbidden outputs',
    meaning: 'Exact lines to disallow (case-sensitive exact per line).',
    options: 'List of strings',
    def: '[]',
  },
  {
    key: 'par',
    setting: 'Parallel chromosomes',
    meaning: 'Max concurrent evaluations.',
    options: 'Integer ≥ 1',
    def: '4',
  },
];

export default function GATLAMHelp() {
  const { setBreadcrumbLabel } = useBreadcrumbContext();
  const ids = useMemo(() => toc.map((t) => t.href.slice(1)), []);
  const { setValue, setBackTo } = useViewSlot();

  useEffect(() => {
    setValue(
      <Typography.Text className="text-base font-medium text-gray-900 dark:text-gray-100 truncate">
        GATLAM Config
      </Typography.Text>,
    );
    setBackTo('/help');
  }, []);

  useEffect(() => {
    setBreadcrumbLabel('help/assignments/config/gatlam', 'GATLAM');
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
            is a <b>genetic algorithm</b> search mode for your assignment.
          </li>
          <li>
            It evaluates runs using <b>return codes</b>, optional <b>time bounds</b>,{' '}
            <b>forbidden outputs</b>, and labeled memo checks.
          </li>
          <li>
            For deeper concepts and scoring, see the <a href="/help/ai/gatlam">GATLAM help</a>.
          </li>
        </ul>
      </Card>
    ),
    onMountScrollToHash: true,
  });

  return (
    <Space direction="vertical" size="small" className="w-full !p-0">
      <Title level={2} className="mb-0">
        GATLAM
      </Title>

      <section id="what" className="scroll-mt-24" />
      <Title level={3}>What is GATLAM mode?</Title>
      <Paragraph className="mb-0">
        <GatlamLink tone="inherit" icon={false} underline={false}>
          GATLAM
        </GatlamLink>{' '}
        runs a small genetic algorithm over your defined <b>genes</b> (ranges) to explore
        inputs/configs. Each candidate is judged using your <b>Task checks</b> (valid return codes,
        optional time bound, forbidden outputs) and by comparing labeled sections of stdout to the
        memo. Use this when you want the system to <i>search</i> for failures or improved solutions
        automatically.
      </Paragraph>

      <Alert
        className="mt-3"
        type="info"
        showIcon
        message="Enable the mode"
        description={
          <>
            Set <b>Project → Submission mode</b> to{' '}
            <GatlamLink tone="inherit" icon={false} underline={false}>
              GATLAM
            </GatlamLink>
            . Configuration below only applies when{' '}
            <GatlamLink tone="inherit" icon={false} underline={false}>
              GATLAM
            </GatlamLink>{' '}
            is enabled. See the full guide:{' '}
            <a href="/help/assignments/gatlam">GATLAM help</a>.
          </>
        }
      />

      <section id="options" className="scroll-mt-24" />
      <Title level={3}>Options & defaults</Title>

      {/* md+ : normal table */}
      <div className="hidden md:block">
        <Table
          size="small"
          columns={optionCols}
          dataSource={optionRows}
          pagination={false}
          scroll={{ x: true }}
        />
      </div>

      {/* <md : cards (no extra shadows) */}
      <div className="block md:hidden !space-y-3">
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
              Options / Format
            </div>
            <div className="text-sm text-gray-900 dark:text-gray-100 mb-2">
              {r.options ?? <span className="text-gray-500">—</span>}
            </div>

            <div className="text-xs uppercase tracking-wide text-gray-500 dark:text-gray-400 mb-1">
              Default
            </div>
            <div className="text-sm text-gray-900 dark:text-gray-100">{r.def}</div>
          </Card>
        ))}
      </div>

      <section id="tips" className="scroll-mt-24" />
      <Title level={3}>Tips</Title>
      <ul className="list-disc pl-5">
        <li>
          Start with defaults; raise <b>Population</b> or <b>Generations</b> only if search stalls.
        </li>
        <li>
          Keep <b>genes</b> realistic — tight ranges focus the search and finish faster.
        </li>
        <li>
          Use <b>Time bound (ms)</b> and <b>Forbidden outputs</b> to encode “bad behavior” simply.
        </li>
        <li>
          Increase <b>Parallel chromosomes</b> cautiously — it uses more compute.
        </li>
      </ul>

      <section id="json" className="scroll-mt-24" />
      <Title level={3}>Raw config (JSON)</Title>
      <Paragraph className="mb-2">
        The UI manages these fields. For reference, JSON keys map to the labels above (e.g.,{' '}
        <Text code>population_size</Text> → Population size).
      </Paragraph>
      <Card>
        <Paragraph className="mb-2">Defaults:</Paragraph>
        <CodeEditor
          language="json"
          value={DEFAULTS_JSON}
          height={280}
          readOnly
          minimal
          fitContent
          showLineNumbers={false}
          hideCopyButton
        />
        <Paragraph className="mt-4 mb-2">Example (slightly larger search + time bound):</Paragraph>
        <CodeEditor
          language="json"
          value={EXAMPLE_JSON}
          height={300}
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
            label: 'No improvements across generations',
            children: (
              <ul className="list-disc pl-5">
                <li>
                  Widen gene ranges or increase <b>Population</b>/<b>Generations</b>.
                </li>
                <li>
                  Raise <b>Mutation probability</b> slightly (e.g., 0.01 → 0.02).
                </li>
              </ul>
            ),
          },
          {
            key: 't2',
            label: 'Everything fails immediately',
            children: (
              <ul className="list-disc pl-5">
                <li>
                  Check <b>Valid return codes</b> include your success code(s).
                </li>
                <li>
                  Relax the <b>Time bound</b> if normal runs exceed it.
                </li>
                <li>
                  Review <b>Forbidden outputs</b> for accidental exact matches.
                </li>
              </ul>
            ),
          },
          {
            key: 't3',
            label: 'Runs are slow',
            children: (
              <ul className="list-disc pl-5">
                <li>
                  Lower <b>Population</b> or <b>Generations</b>, or reduce{' '}
                  <b>Parallel chromosomes</b>.
                </li>
                <li>Trim task output/log noise to speed comparisons.</li>
              </ul>
            ),
          },
        ]}
      />
    </Space>
  );
}
