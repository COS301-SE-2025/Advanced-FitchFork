// StepConfig.tsx

import { useEffect, useMemo, useState } from 'react';
import {
  Collapse,
  type CollapseProps,
  Form,
  InputNumber,
  Input,
  Switch,
  Segmented,
  Button,
  Space,
  Typography,
  Divider,
  Select,
} from 'antd';
import { useModule } from '@/context/ModuleContext';
import { useAssignmentSetup } from '@/context/AssignmentSetupContext';
import { resetAssignmentConfig, setAssignmentConfig } from '@/services/modules/assignments/config';
import {
  LANGUAGE_OPTIONS,
  SUBMISSION_MODE_OPTIONS,
  MARKING_SCHEME_OPTIONS,
  FEEDBACK_SCHEME_OPTIONS,
  GRADING_POLICY_OPTIONS,
  CROSSOVER_TYPE_OPTIONS,
  MUTATION_TYPE_OPTIONS,
} from '@/types/modules/assignments/config';
import type { AssignmentConfig } from '@/types/modules/assignments/config';
import Tip from '@/components/common/Tip';

const { Title, Paragraph, Text } = Typography;

// --- Defaults matching backend ExecutionConfig::default_config() ---
const DEFAULT_CONFIG: AssignmentConfig = {
  project: { language: 'cpp', submission_mode: 'manual' },
  execution: {
    timeout_secs: 10,
    max_memory: 8_589_934_592,
    max_cpus: 2,
    max_uncompressed_size: 100_000_000,
    max_processes: 256,
  },
  marking: {
    marking_scheme: 'exact',
    feedback_scheme: 'auto',
    deliminator: '&-=-&',
    grading_policy: 'last',
    max_attempts: 10,
    limit_attempts: false,
    pass_mark: 50,
    allow_practice_submissions: false,
    dissalowed_code: [],
  },
  output: { stdout: true, stderr: false, retcode: false },
  gatlam: {
    population_size: 100,
    number_of_generations: 50,
    selection_size: 20,
    reproduction_probability: 0.8,
    crossover_probability: 0.9,
    mutation_probability: 0.01,
    genes: [
      { min_value: -5, max_value: 5 },
      { min_value: -4, max_value: 9 },
    ],
    crossover_type: 'onepoint',
    mutation_type: 'bitflip',
    omega1: 0.5,
    omega2: 0.3,
    omega3: 0.2,
    task_spec: { valid_return_codes: [0], max_runtime_ms: undefined, forbidden_outputs: [] },
    max_parallel_chromosomes: 4,
    verbose: false,
  },
  security: {
    password_enabled: false,
    password_pin: null,
    cookie_ttl_minutes: 480,
    bind_cookie_to_user: true,
    allowed_cidrs: [],
  },
  code_coverage: { code_coverage_weight: 10 },
};

// Merge “cfg” over defaults (prevents undefined access)
function hydrateConfig(cfg?: Partial<AssignmentConfig>): AssignmentConfig {
  const base = DEFAULT_CONFIG;
  return {
    project: { ...base.project, ...(cfg?.project ?? {}) },
    execution: { ...base.execution, ...(cfg?.execution ?? {}) },
    marking: { ...base.marking, ...(cfg?.marking ?? {}) },
    output: { ...base.output, ...(cfg?.output ?? {}) },
    gatlam: {
      ...base.gatlam,
      ...(cfg?.gatlam ?? {}),
      task_spec: { ...base.gatlam.task_spec, ...(cfg?.gatlam?.task_spec ?? {}) },
    },
    security: { ...base.security, ...(cfg?.security ?? {}) },
    code_coverage: { ...base.code_coverage, ...(cfg?.code_coverage ?? {}) },
  };
}

const StepConfig = () => {
  const module = useModule();
  const { assignmentId, config, setConfig, refreshAssignment, setStepSaveHandler, next } =
    useAssignmentSetup();

  const [form] = Form.useForm();
  const [saving, setSaving] = useState(false);

  // If config is missing, create defaults on backend once; otherwise hydrate safely.
  useEffect(() => {
    (async () => {
      if (!assignmentId) return;
      if (!config) {
        const res = await resetAssignmentConfig(module.id, assignmentId);
        if (res.success) {
          setConfig(res.data);
          await refreshAssignment?.();
        } else {
          // fallback: allow editing hydrated defaults locally
          setConfig(DEFAULT_CONFIG);
        }
      }
    })();
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [assignmentId, config]);

  const safeConfig = useMemo(() => hydrateConfig(config ?? undefined), [config]);

  // Populate form when config (or hydration) changes
  useEffect(() => {
    const c = safeConfig;
    form.setFieldsValue({
      // Project
      language: c.project.language,
      submission_mode: c.project.submission_mode,
      // Execution
      timeout_secs: c.execution.timeout_secs,
      max_memory: c.execution.max_memory,
      max_cpus: c.execution.max_cpus,
      max_uncompressed_size: c.execution.max_uncompressed_size,
      max_processes: c.execution.max_processes,
      // Marking
      marking_scheme: c.marking.marking_scheme,
      feedback_scheme: c.marking.feedback_scheme,
      grading_policy: c.marking.grading_policy,
      deliminator: c.marking.deliminator,
      max_attempts: c.marking.max_attempts,
      limit_attempts: c.marking.limit_attempts,
      pass_mark: c.marking.pass_mark,
      allow_practice_submissions: c.marking.allow_practice_submissions,
      dissalowed_code: c.marking.dissalowed_code ?? [],
      // Output
      stdout: c.output.stdout,
      stderr: c.output.stderr,
      retcode: c.output.retcode,
      // Security
      password_enabled: c.security.password_enabled,
      password_pin: c.security.password_pin ?? '',
      cookie_ttl_minutes: c.security.cookie_ttl_minutes,
      bind_cookie_to_user: c.security.bind_cookie_to_user,
      allowed_cidrs: c.security.allowed_cidrs ?? [],
      // GATLAM (subset commonly tuned)
      population_size: c.gatlam.population_size,
      number_of_generations: c.gatlam.number_of_generations,
      selection_size: c.gatlam.selection_size,
      reproduction_probability: c.gatlam.reproduction_probability,
      crossover_probability: c.gatlam.crossover_probability,
      mutation_probability: c.gatlam.mutation_probability,
      crossover_type: c.gatlam.crossover_type,
      mutation_type: c.gatlam.mutation_type,
      omega1: c.gatlam.omega1,
      omega2: c.gatlam.omega2,
      omega3: c.gatlam.omega3,
      verbose: c.gatlam.verbose,
      max_parallel_chromosomes: c.gatlam.max_parallel_chromosomes,
      // Coverage
      code_coverage_required: c.code_coverage.code_coverage_weight,
    });
  }, [safeConfig, form]);

  useEffect(() => {
    setStepSaveHandler?.(1, async () => true);
  }, [setStepSaveHandler]);

  const onResetToDefaults = async () => {
    if (!assignmentId) return;
    setSaving(true);
    try {
      const res = await resetAssignmentConfig(module.id, assignmentId);
      const fresh = res.success ? res.data : DEFAULT_CONFIG;
      setConfig(fresh);
      await refreshAssignment?.();
    } finally {
      setSaving(false);
    }
  };

  const onSave = async (advance = false) => {
    if (!assignmentId) return;
    const c = safeConfig; // base
    try {
      setSaving(true);
      const v = await form.validateFields();

      // Normalize lists from multi-selects
      const dissalowed_code: string[] = (v.dissalowed_code || [])
        .map((s: any) => String(s).trim())
        .filter(Boolean);

      const allowed_cidrs: string[] = (v.allowed_cidrs || [])
        .map((s: any) => String(s).trim())
        .filter(Boolean);

      const updated: AssignmentConfig = {
        ...c,
        project: { ...c.project, language: v.language, submission_mode: v.submission_mode },
        execution: {
          ...c.execution,
          timeout_secs: v.timeout_secs,
          max_memory: v.max_memory,
          max_cpus: v.max_cpus,
          max_uncompressed_size: v.max_uncompressed_size,
          max_processes: v.max_processes,
        },
        marking: {
          ...c.marking,
          marking_scheme: v.marking_scheme,
          feedback_scheme: v.feedback_scheme,
          grading_policy: v.grading_policy,
          deliminator: v.deliminator,
          max_attempts: v.max_attempts,
          limit_attempts: v.limit_attempts,
          pass_mark: v.pass_mark,
          allow_practice_submissions: v.allow_practice_submissions,
          dissalowed_code,
        },
        output: { ...c.output, stdout: v.stdout, stderr: v.stderr, retcode: v.retcode },
        security: {
          ...c.security,
          password_enabled: v.password_enabled,
          password_pin: v.password_pin ? String(v.password_pin) : null,
          cookie_ttl_minutes: v.cookie_ttl_minutes,
          bind_cookie_to_user: v.bind_cookie_to_user,
          allowed_cidrs,
        },
        gatlam: {
          ...c.gatlam,
          population_size: v.population_size,
          number_of_generations: v.number_of_generations,
          selection_size: v.selection_size,
          reproduction_probability: v.reproduction_probability,
          crossover_probability: v.crossover_probability,
          mutation_probability: v.mutation_probability,
          crossover_type: v.crossover_type,
          mutation_type: v.mutation_type,
          omega1: v.omega1,
          omega2: v.omega2,
          omega3: v.omega3,
          task_spec: { ...c.gatlam.task_spec }, // unchanged
          verbose: v.verbose,
          max_parallel_chromosomes: v.max_parallel_chromosomes,
        },
        code_coverage: { ...c.code_coverage, code_coverage_weight: v.code_coverage_required },
      };

      const res = await setAssignmentConfig(module.id, assignmentId, updated);
      setConfig(res.data ?? updated);
      await refreshAssignment?.();
      if (advance) await next?.();
    } finally {
      setSaving(false);
    }
  };

  // Helper to render a section label with a subtle help icon
  const sectionLabel = (title: string, helpHref: string, tipText: string) => (
    <span className="inline-flex items-center gap-2">
      <span>{title}</span>
      <Tip iconOnly newTab to={helpHref} text={tipText} />
    </span>
  );

  // Panels UI (accordions)
  const panels: CollapseProps['items'] = [
    {
      key: 'project',
      label: sectionLabel('Project', '/help/assignments/config/project', 'Project config help'),
      children: (
        <Space direction="vertical" size="large" className="w-full">
          <Typography.Paragraph type="secondary" className="!mb-0">
            Choose your language and how students submit. Manual expects Main files; GATLAM expects
            an Interpreter.
          </Typography.Paragraph>
          <Form.Item name="language" label="Language" rules={[{ required: true }]}>
            <Select
              showSearch
              style={{ maxWidth: 320 }}
              options={LANGUAGE_OPTIONS.map((o) => ({
                label: o.label,
                value: o.value,
                // Auto-disable anything labeled "(not implemented)"
                disabled: /\(not implemented\)/i.test(String(o.label)),
              }))}
              filterOption={(input, option) =>
                String(option?.label ?? '')
                  .toLowerCase()
                  .includes(input.toLowerCase())
              }
              optionFilterProp="label"
              placeholder="Select a language"
            />
          </Form.Item>

          <Form.Item name="submission_mode" label="Submission Mode" rules={[{ required: true }]}>
            <Segmented
              options={SUBMISSION_MODE_OPTIONS.map((o) => ({ label: o.label, value: o.value }))}
            />
          </Form.Item>

          <Text type="secondary" className="text-xs">
            Tip: <b>manual</b> expects Main files; <b>gatlam</b> expects an Interpreter.
          </Text>
        </Space>
      ),
    },
    {
      key: 'execution',
      label: sectionLabel(
        'Execution Limits',
        '/help/assignments/config/execution',
        'Execution limits help',
      ),
      children: (
        <Space direction="vertical" size="large" className="w-full">
          <Typography.Paragraph type="secondary" className="!mb-0">
            Constrain time, memory, processes and archive sizes to keep runs safe and fair.
          </Typography.Paragraph>
          <Space wrap size="large">
            <Form.Item name="timeout_secs" label="Timeout (s)" rules={[{ required: true }]}>
              <InputNumber min={1} />
            </Form.Item>
            <Form.Item name="max_memory" label="Max Memory (bytes)" rules={[{ required: true }]}>
              <InputNumber min={1024 * 1024} step={1024 * 1024} />
            </Form.Item>
            <Form.Item name="max_cpus" label="Max CPUs" rules={[{ required: true }]}>
              <InputNumber min={1} />
            </Form.Item>
            <Form.Item
              name="max_uncompressed_size"
              label="Max Uncompressed Size (bytes)"
              rules={[{ required: true }]}
            >
              <InputNumber min={1024 * 1024} step={1024 * 1024} />
            </Form.Item>
            <Form.Item name="max_processes" label="Max Processes" rules={[{ required: true }]}>
              <InputNumber min={1} />
            </Form.Item>
          </Space>
        </Space>
      ),
    },
    {
      key: 'marking',
      label: sectionLabel('Marking', '/help/assignments/config/marking', 'Marking config help'),
      children: (
        <>
          <Space direction="vertical" size="large" className="w-full">
            <Typography.Paragraph type="secondary" className="!mb-0">
              Choose how outputs are compared and which feedback students see. Set your delimiter to
              match labels in program output.
            </Typography.Paragraph>
            <Space wrap size="large">
              <Form.Item name="marking_scheme" label="Marking Scheme" rules={[{ required: true }]}>
                <Segmented
                  options={MARKING_SCHEME_OPTIONS.map((o) => ({ label: o.label, value: o.value }))}
                />
              </Form.Item>
              <Form.Item name="feedback_scheme" label="Feedback" rules={[{ required: true }]}>
                <Segmented
                  options={FEEDBACK_SCHEME_OPTIONS.map((o) => ({ label: o.label, value: o.value }))}
                />
              </Form.Item>
              <Form.Item name="grading_policy" label="Grading Policy" rules={[{ required: true }]}>
                <Segmented
                  options={GRADING_POLICY_OPTIONS.map((o) => ({ label: o.label, value: o.value }))}
                />
              </Form.Item>
            </Space>

            <Divider />
            <Space wrap size="large">
              <Form.Item name="deliminator" label="Output Delimiter" rules={[{ required: true }]}>
                <Input placeholder="&-=-&" />
              </Form.Item>
              <Form.Item name="pass_mark" label="Pass Mark (%)" rules={[{ required: true }]}>
                <InputNumber min={0} max={100} />
              </Form.Item>
            </Space>

            <Space wrap size="large" align="center">
              <Form.Item
                name="limit_attempts"
                label="Limit Attempts"
                valuePropName="checked"
                tooltip="If false, attempt limits are not enforced."
              >
                <Switch />
              </Form.Item>
              <Form.Item name="max_attempts" label="Max Attempts">
                <InputNumber min={1} />
              </Form.Item>
              <Form.Item
                name="allow_practice_submissions"
                label="Allow Practice Submissions"
                valuePropName="checked"
              >
                <Switch />
              </Form.Item>
            </Space>

            <Form.Item name="dissalowed_code" label="Disallowed Code">
              <Select
                mode="tags"
                tokenSeparators={[',', ' ']}
                placeholder="Add substrings to block (e.g., system(, exec() )"
                style={{ width: '100%' }}
              />
            </Form.Item>
          </Space>
        </>
      ),
    },
    {
      key: 'output',
      label: sectionLabel('Execution Output', '/help/assignments/config/output', 'Output help'),
      children: (
        <Space direction="vertical" size="large" className="w-full">
          <Typography.Paragraph type="secondary" className="!mb-0">
            Choose which streams to capture and compare against Memo Output.
          </Typography.Paragraph>
          <Space wrap size="large">
            <Form.Item name="stdout" label="Capture stdout" valuePropName="checked">
              <Switch />
            </Form.Item>
            <Form.Item name="stderr" label="Capture stderr" valuePropName="checked">
              <Switch />
            </Form.Item>
            <Form.Item name="retcode" label="Include return code" valuePropName="checked">
              <Switch />
            </Form.Item>
          </Space>
        </Space>
      ),
    },
    {
      key: 'security',
      label: sectionLabel('Security', '/help/assignments/config/security', 'Security help'),
      children: (
        <Space direction="vertical" size="large" className="w-full">
          <Typography.Paragraph type="secondary" className="!mb-0">
            Lock an assignment behind a PIN, limit access by IP ranges, and bind cookies to users.
          </Typography.Paragraph>
          <Space wrap size="large" align="center">
            <Form.Item name="password_enabled" label="Require Unlock" valuePropName="checked">
              <Switch />
            </Form.Item>
            <Form.Item name="password_pin" label="PIN (optional)">
              <Input placeholder="1234" />
            </Form.Item>
            <Form.Item
              name="cookie_ttl_minutes"
              label="Cookie TTL (minutes)"
              rules={[{ required: true }]}
            >
              <InputNumber min={1} />
            </Form.Item>
            <Form.Item
              name="bind_cookie_to_user"
              label="Bind Cookie to User"
              valuePropName="checked"
            >
              <Switch />
            </Form.Item>
          </Space>
          <Form.Item name="allowed_cidrs" label="Allowed CIDRs (empty = allow all)">
            <Select
              mode="tags"
              tokenSeparators={[',', ' ']}
              placeholder="e.g., 10.0.0.0/24, 196.21.0.0/16"
              style={{ width: '100%' }}
            />
          </Form.Item>
        </Space>
      ),
    },
    {
      key: 'gatlam',
      label: sectionLabel('GATLAM', '/help/assignments/config/gatlam', 'GATLAM config help'),
      children: (
        <Space direction="vertical" size="large" className="w-full">
          <Typography.Paragraph type="secondary" className="!mb-0">
            Configure the genetic algorithm used for automated evaluation in GATLAM mode.
          </Typography.Paragraph>
          <Space wrap size="large">
            <Form.Item name="population_size" label="Population" rules={[{ required: true }]}>
              <InputNumber min={1} />
            </Form.Item>
            <Form.Item
              name="number_of_generations"
              label="Generations"
              rules={[{ required: true }]}
            >
              <InputNumber min={1} />
            </Form.Item>
            <Form.Item name="selection_size" label="Selection Size" rules={[{ required: true }]}>
              <InputNumber min={1} />
            </Form.Item>
          </Space>

          <Space wrap size="large">
            <Form.Item
              name="reproduction_probability"
              label="Reproduction p"
              rules={[{ required: true }]}
            >
              <InputNumber min={0} max={1} step={0.01} />
            </Form.Item>
            <Form.Item
              name="crossover_probability"
              label="Crossover p"
              rules={[{ required: true }]}
            >
              <InputNumber min={0} max={1} step={0.01} />
            </Form.Item>
            <Form.Item name="mutation_probability" label="Mutation p" rules={[{ required: true }]}>
              <InputNumber min={0} max={1} step={0.01} />
            </Form.Item>
          </Space>

          <Space wrap size="large">
            <Form.Item name="crossover_type" label="Crossover" rules={[{ required: true }]}>
              <Segmented
                options={CROSSOVER_TYPE_OPTIONS.map((o) => ({
                  label: o.label,
                  value: o.value,
                }))}
              />
            </Form.Item>
            <Form.Item name="mutation_type" label="Mutation" rules={[{ required: true }]}>
              <Segmented
                options={MUTATION_TYPE_OPTIONS.map((o) => ({
                  label: o.label,
                  value: o.value,
                }))}
              />
            </Form.Item>
          </Space>

          <Space wrap size="large">
            <Form.Item name="omega1" label="ω1" rules={[{ required: true }]}>
              <InputNumber step={0.01} />
            </Form.Item>
            <Form.Item name="omega2" label="ω2" rules={[{ required: true }]}>
              <InputNumber step={0.01} />
            </Form.Item>
            <Form.Item name="omega3" label="ω3" rules={[{ required: true }]}>
              <InputNumber step={0.01} />
            </Form.Item>
          </Space>

          <Space wrap size="large" align="center">
            <Form.Item name="verbose" label="Verbose" valuePropName="checked">
              <Switch />
            </Form.Item>
            <Form.Item
              name="max_parallel_chromosomes"
              label="Max Parallel Chromosomes"
              rules={[{ required: true }]}
            >
              <InputNumber min={1} />
            </Form.Item>
          </Space>

          <Text type="secondary" className="text-xs">
            Genes and task spec are preserved as-is; advanced editors can be added later.
          </Text>
        </Space>
      ),
    },
    {
      key: 'coverage',
      label: sectionLabel('Code Coverage', '/help/assignments/code-coverage', 'Coverage help'),
      children: (
        <Space direction="vertical" size="large" className="w-full">
          <Typography.Paragraph type="secondary" className="!mb-0">
            Set your coverage target for assignments that measure code coverage.
          </Typography.Paragraph>
          <Form.Item
            name="code_coverage_required"
            label="Required Coverage (%)"
            rules={[{ required: true }]}
          >
            <InputNumber min={0} max={100} />
          </Form.Item>
        </Space>
      ),
    },
  ];

  return (
    <div className="space-y-6">
      <div className="flex items-center gap-2">
        <Title level={3} className="!mb-0">
          Assignment Configuration
        </Title>
        <Tip iconOnly newTab to="/help/assignments/config/overview" text="Config help" />
      </div>
      <Paragraph type="secondary" className="!mb-2">
        Defaults are applied automatically. Tweak any section below. Save in place or{' '}
        <b>Save &amp; Continue</b> to move on.
      </Paragraph>

      <Form form={form} layout="vertical">
        <Collapse
          items={panels}
          accordion
          defaultActiveKey="project" // only one panel open at a time
        />

        <div className="flex justify-end gap-2 pt-4">
          <Button onClick={onResetToDefaults} disabled={saving}>
            Reset to Defaults
          </Button>
          <Button type="default" onClick={() => onSave(false)} loading={saving}>
            Save
          </Button>
          <Button type="primary" onClick={() => onSave(true)} loading={saving}>
            Save &amp; Continue
          </Button>
        </div>
      </Form>
    </div>
  );
};

export default StepConfig;
