import { useCallback, useEffect, useState } from 'react';
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
import { GatlamLink } from '@/components/common';

const { Title, Paragraph, Text } = Typography;

const StepConfig = () => {
  const module = useModule();
  const { assignmentId, config, setConfig, refreshAssignment, setStepSaveHandler } =
    useAssignmentSetup();

  const [form] = Form.useForm();
  const [saving, setSaving] = useState(false);

  // Initialize on first mount if there's no config: ask server to reset/create default.
  useEffect(() => {
    (async () => {
      if (!assignmentId) return;
      if (!config) {
        const res = await resetAssignmentConfig(module.id, assignmentId);
        if (res.success) {
          setConfig(res.data);
          await refreshAssignment?.();
        } else {
          // leave config as null; the UI will be empty until backend is reachable
          // (intentionally no client-side defaults)
          // console.error('resetAssignmentConfig failed:', res.message);
        }
      }
    })();
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [assignmentId, config]);

  // Populate form whenever server-backed config changes
  useEffect(() => {
    if (!config) return;

    const c = config;
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
      // NEW: Late policy
      allow_late_submissions: c.marking.late?.allow_late_submissions ?? false,
      late_window_minutes: c.marking.late?.late_window_minutes ?? 0,
      late_max_percent: c.marking.late?.late_max_percent ?? 100,
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
      code_coverage_whitelist: c.code_coverage.whitelist ?? [],
    });
  }, [config, form]);

  const onResetToDefaults = async () => {
    if (!assignmentId) return;
    setSaving(true);
    try {
      const res = await resetAssignmentConfig(module.id, assignmentId);
      if (res.success) {
        setConfig(res.data);
        await refreshAssignment?.();
        form.resetFields(); // effect above will repopulate
      } else {
        // console.error('resetAssignmentConfig failed:', res.message);
      }
    } finally {
      setSaving(false);
    }
  };

  const persistConfig = useCallback(
    async ({ skipRefresh = false }: { skipRefresh?: boolean } = {}) => {
      if (!assignmentId || !config) return false;
      setSaving(true);
      try {
        const v = await form.validateFields();

        // Normalize lists from multi-selects
        const dissalowed_code: string[] = (v.dissalowed_code || [])
          .map((s: any) => String(s).trim())
          .filter(Boolean);

        const allowed_cidrs: string[] = (v.allowed_cidrs || [])
          .map((s: any) => String(s).trim())
          .filter(Boolean);

        const code_coverage_whitelist: string[] = (v.code_coverage_whitelist || [])
          .map((s: any) => String(s).trim())
          .filter(Boolean);

        const c = config; // use current server-backed object as base

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
            late: {
              allow_late_submissions: !!v.allow_late_submissions,
              late_window_minutes: Number(v.late_window_minutes ?? 0),
              late_max_percent: Number(v.late_max_percent ?? 100),
            },
          },
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
            task_spec: { ...c.gatlam.task_spec }, // unchanged here
            verbose: v.verbose,
            max_parallel_chromosomes: v.max_parallel_chromosomes,
          },
          code_coverage: {
            ...c.code_coverage,
            code_coverage_weight: v.code_coverage_required,
            whitelist: code_coverage_whitelist,
          },
        };

        const res = await setAssignmentConfig(module.id, assignmentId, updated);
        setConfig(res.data ?? updated);
        if (!skipRefresh) await refreshAssignment?.();
        return true;
      } catch (err) {
        return false;
      } finally {
        setSaving(false);
      }
    },
    [assignmentId, config, form, module.id, refreshAssignment, setConfig],
  );

  const onSave = useCallback(async () => persistConfig({ skipRefresh: false }), [persistConfig]);

  const stepSaveHandler = useCallback(() => persistConfig({ skipRefresh: true }), [persistConfig]);

  useEffect(() => {
    if (!setStepSaveHandler) return;
    setStepSaveHandler(1, stepSaveHandler);
  }, []);

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
            Choose your language and how students submit. Manual expects Main files;{' '}
            <GatlamLink tone="inherit" icon={false} underline={false}>
              GATLAM
            </GatlamLink>{' '}
            expects an Interpreter.
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
            Tip: <b>manual</b> expects Main files;{' '}
            <GatlamLink tone="inherit" icon={false} underline={false}>
              gatlam
            </GatlamLink>{' '}
            expects an Interpreter.
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
                <Input placeholder="###" />
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

            <Divider />
            <Typography.Paragraph strong className="!mb-2">
              Late Submission Policy
            </Typography.Paragraph>
            <Space wrap size="large" align="center">
              <Form.Item name="allow_late_submissions" label="Allow Late" valuePropName="checked">
                <Switch />
              </Form.Item>
              <Form.Item name="late_window_minutes" label="Late Window (min)">
                <InputNumber min={0} />
              </Form.Item>
              <Form.Item name="late_max_percent" label="Late Max (%)">
                <InputNumber min={0} max={100} />
              </Form.Item>
            </Space>
          </Space>
        </>
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
            Configure the genetic algorithm used for automated evaluation in{' '}
            <GatlamLink tone="inherit" icon={false} underline={false}>
              GATLAM
            </GatlamLink>{' '}
            mode.
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
            Set your coverage target and optional whitelist for files/packages included in coverage.
          </Typography.Paragraph>
          <Space wrap size="large">
            <Form.Item
              name="code_coverage_required"
              label="Required Coverage (%)"
              rules={[{ required: true }]}
            >
              <InputNumber min={0} max={100} />
            </Form.Item>
            <Form.Item
              name="code_coverage_whitelist"
              label="Coverage Whitelist"
              tooltip="Optional list of file names that should be counted when calculating coverage. Leave empty to include all files."
            >
              <Select
                mode="tags"
                tokenSeparators={[',', ' ']}
                placeholder="e.g., main.cpp, utils.c, MyClass.java"
                style={{ minWidth: 360 }}
              />
            </Form.Item>
          </Space>
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
        Defaults are applied on the server. Tweak any section below. Save in place or use{' '}
        <b>Next</b> to save and continue.
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
          <Button type="default" onClick={() => void onSave()} loading={saving}>
            Save
          </Button>
        </div>
      </Form>
    </div>
  );
};

export default StepConfig;
