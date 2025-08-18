import { useEffect, useMemo } from 'react';
import {
  Typography,
  Form,
  InputNumber,
  Select,
  Switch,
  Button,
  Space,
  Divider,
  Input,
  Tag,
  Tooltip,
} from 'antd';
import { PlusOutlined, DeleteOutlined, InfoCircleOutlined } from '@ant-design/icons';

import SettingsGroup from '@/components/SettingsGroup';
import { useViewSlot } from '@/context/ViewSlotContext';
import { useAssignment } from '@/context/AssignmentContext';
import { message } from '@/utils/message';

import {
  CROSSOVER_TYPE_OPTIONS,
  MUTATION_TYPE_OPTIONS,
  type GatlamConfig,
} from '@/types/modules/assignments/config';

import AssignmentConfigActions from '@/components/assignments/AssignmentConfigActions';
import { useUI } from '@/context/UIContext';

const clamp01 = (n: number) => Math.max(0, Math.min(1, n));
const round2 = (n: number) => Math.round(n * 100) / 100;

export default function GatlamPage() {
  const { isSm } = useUI();
  const { setValue } = useViewSlot();
  const { config, updateConfig } = useAssignment();
  const [form] = Form.useForm<GatlamConfig>();

  useEffect(() => {
    setValue(
      <Typography.Text className="text-base font-medium text-gray-900 dark:text-gray-100 truncate">
        GA / TLAM Configuration
      </Typography.Text>,
    );
  }, [setValue]);

  // Seed form from context whenever GATLAM changes
  useEffect(() => {
    if (!config?.gatlam) return;
    form.setFieldsValue(config.gatlam);
  }, [config?.gatlam, form]);

  // ---- Live sum & helpers for weights --------------------------------------
  const w1 = Form.useWatch('omega1', form) ?? 0;
  const w2 = Form.useWatch('omega2', form) ?? 0;
  const w3 = Form.useWatch('omega3', form) ?? 0;

  const sum = useMemo(() => w1 + w2 + w3, [w1, w2, w3]);

  const setWeightsNormalized = (key: 'omega1' | 'omega2' | 'omega3', nextVal: number | null) => {
    const v = clamp01(Number(nextVal ?? 0));
    const current = {
      omega1: Number(form.getFieldValue('omega1') ?? 0),
      omega2: Number(form.getFieldValue('omega2') ?? 0),
      omega3: Number(form.getFieldValue('omega3') ?? 0),
    };

    current[key] = v;

    const keys = ['omega1', 'omega2', 'omega3'] as const;
    const others = keys.filter((k) => k !== key);
    const residual = 1 - v;

    let a = current[others[0]];
    let b = current[others[1]];
    const ab = a + b;

    if (ab <= 0) {
      a = residual / 2;
      b = residual / 2;
    } else {
      const scale = residual / ab;
      a *= scale;
      b *= scale;
    }

    form.setFieldsValue({
      [key]: v,
      [others[0]]: a,
      [others[1]]: b,
    });
  };

  const equalize = () => {
    form.setFieldsValue({ omega1: 1 / 3, omega2: 1 / 3, omega3: 1 / 3 });
  };

  const bias = (key: 'omega1' | 'omega2' | 'omega3', amount = 0.6) => {
    const rest = (1 - amount) / 2;
    const vals = { omega1: rest, omega2: rest, omega3: rest } as Record<
      'omega1' | 'omega2' | 'omega3',
      number
    >;
    vals[key] = amount;
    form.setFieldsValue(vals);
  };

  const onSave = async () => {
    if (!config) {
      message.error('No configuration loaded yet.');
      return;
    }
    const values = await form.validateFields(); // GatlamConfig
    const S = values.omega1 + values.omega2 + values.omega3;
    if (Math.abs(S - 1) > 1e-6) {
      values.omega1 = values.omega1 / S;
      values.omega2 = values.omega2 / S;
      values.omega3 = values.omega3 / S;
    }

    try {
      await updateConfig({ gatlam: values });
      message.success('GATLAM config saved');
    } catch (e: any) {
      message.error(e?.message ?? 'Failed to save GATLAM config');
    }
  };

  const fieldWidth = 'w-full sm:max-w-xs';
  const disabled = !config;

  return (
    <div className="flex flex-col gap-4">
      <Form form={form} layout="vertical" disabled={disabled}>
        <Space direction="vertical" size="large" className="w-full">
          {/* ---- GA Core ---- */}
          <SettingsGroup
            title="Genetic Algorithm"
            description="Control population dynamics, selection, and genetic operators."
          >
            <Form.Item
              name="population_size"
              label="Population Size"
              className={fieldWidth}
              rules={[{ required: true }]}
            >
              <InputNumber min={1} step={1} precision={0} className="w-full" />
            </Form.Item>

            <Form.Item
              name="number_of_generations"
              label="Generations"
              className={fieldWidth}
              rules={[{ required: true }]}
            >
              <InputNumber min={1} step={1} precision={0} className="w-full" />
            </Form.Item>

            <Form.Item
              name="selection_size"
              label="Selection Size"
              className={fieldWidth}
              rules={[{ required: true }]}
            >
              <InputNumber min={1} step={1} precision={0} className="w-full" />
            </Form.Item>

            <Form.Item
              name="reproduction_probability"
              label="Reproduction Prob."
              className={fieldWidth}
              rules={[{ required: true }]}
            >
              <InputNumber min={0} max={1} step={0.01} className="w-full" />
            </Form.Item>

            <Form.Item
              name="crossover_probability"
              label="Crossover Prob."
              className={fieldWidth}
              rules={[{ required: true }]}
            >
              <InputNumber min={0} max={1} step={0.01} className="w-full" />
            </Form.Item>

            <Form.Item
              name="mutation_probability"
              label="Mutation Prob."
              className={fieldWidth}
              rules={[{ required: true }]}
            >
              <InputNumber min={0} max={1} step={0.001} className="w-full" />
            </Form.Item>

            <Form.Item
              name="crossover_type"
              label="Crossover Type"
              className={fieldWidth}
              rules={[{ required: true }]}
            >
              <Select className="w-full" options={CROSSOVER_TYPE_OPTIONS} />
            </Form.Item>

            <Form.Item
              name="mutation_type"
              label="Mutation Type"
              className={fieldWidth}
              rules={[{ required: true }]}
            >
              <Select className="w-full" options={MUTATION_TYPE_OPTIONS} />
            </Form.Item>
          </SettingsGroup>

          {/* ---- Genes ---- */}
          <SettingsGroup
            title="Genes"
            description="Define the value ranges for each gene in a chromosome."
          >
            <Form.List name="genes">
              {(fields, { add, remove }) => (
                <Space direction="vertical" className="w-full">
                  {fields.map((field) => (
                    <div key={field.key} className="w-full">
                      {isSm ? (
                        // >= sm: Compact row
                        <Space.Compact className="w-full">
                          <Form.Item
                            name={[field.name, 'min_value']}
                            noStyle
                            rules={[{ required: true }]}
                          >
                            <InputNumber className="w-full" placeholder="Min" />
                          </Form.Item>
                          <Form.Item
                            name={[field.name, 'max_value']}
                            noStyle
                            rules={[{ required: true }]}
                          >
                            <InputNumber className="w-full" placeholder="Max" />
                          </Form.Item>
                          <Button
                            icon={<DeleteOutlined />}
                            onClick={() => remove(field.name)}
                            danger
                          />
                        </Space.Compact>
                      ) : (
                        // < sm: stacked
                        <Space direction="vertical" className="w-full">
                          <Form.Item
                            name={[field.name, 'min_value']}
                            rules={[{ required: true }]}
                            className="!mb-0"
                          >
                            <InputNumber className="w-full" placeholder="Min" />
                          </Form.Item>
                          <Form.Item
                            name={[field.name, 'max_value']}
                            rules={[{ required: true }]}
                            className="!mb-0"
                          >
                            <InputNumber className="w-full" placeholder="Max" />
                          </Form.Item>
                          <div>
                            <Button
                              icon={<DeleteOutlined />}
                              onClick={() => remove(field.name)}
                              danger
                            />
                          </div>
                        </Space>
                      )}
                    </div>
                  ))}
                  <Button onClick={() => add()} icon={<PlusOutlined />}>
                    Add Gene
                  </Button>
                </Space>
              )}
            </Form.List>
          </SettingsGroup>

          {/* ---- Component Weights (sum to 1) ---- */}
          <SettingsGroup
            title={
              <Space>
                Component Weights
                <Tooltip title="Weights are auto-normalized so ω₁ + ω₂ + ω₃ = 1">
                  <InfoCircleOutlined />
                </Tooltip>
              </Space>
            }
            description="Tune the weighted components used in the GA objective. Adjusting one weight will rebalance the others."
          >
            {/* Controls row */}
            <div className="flex flex-wrap items-center gap-2 mb-3">
              <Tag color={Math.abs(sum - 1) < 1e-6 ? 'green' : 'red'}>
                Σ ω = {round2(sum).toFixed(2)}
              </Tag>
              <Space size="small" wrap>
                <Button size="small" onClick={equalize}>
                  Equalize (⅓,⅓,⅓)
                </Button>
                <Button size="small" onClick={() => bias('omega1')}>
                  Bias ω₁
                </Button>
                <Button size="small" onClick={() => bias('omega2')}>
                  Bias ω₂
                </Button>
                <Button size="small" onClick={() => bias('omega3')}>
                  Bias ω₃
                </Button>
              </Space>
            </div>

            {/* Always stacked vertically */}
            <Space direction="vertical" className="w-full">
              <Form.Item
                name="omega1"
                label="ω₁"
                rules={[{ required: true }]}
                className="w-full sm:max-w-xs !mb-0"
              >
                <InputNumber
                  min={0}
                  max={1}
                  step={0.01}
                  className="w-full"
                  onChange={(v) => setWeightsNormalized('omega1', v)}
                />
              </Form.Item>

              <Form.Item
                name="omega2"
                label="ω₂"
                rules={[{ required: true }]}
                className="w-full sm:max-w-xs !mb-0"
              >
                <InputNumber
                  min={0}
                  max={1}
                  step={0.01}
                  className="w-full"
                  onChange={(v) => setWeightsNormalized('omega2', v)}
                />
              </Form.Item>

              <Form.Item
                name="omega3"
                label="ω₃"
                rules={[{ required: true }]}
                className="w-full sm:max-w-xs !mb-0"
              >
                <InputNumber
                  min={0}
                  max={1}
                  step={0.01}
                  className="w-full"
                  onChange={(v) => setWeightsNormalized('omega3', v)}
                />
              </Form.Item>
            </Space>
          </SettingsGroup>

          {/* ---- TaskSpec ---- */}
          <SettingsGroup
            title="Task Specification"
            description="Runtime and validation rules for executing chromosomes."
          >
            <Form.Item
              name={['task_spec', 'max_runtime_ms']}
              label="Max Runtime"
              tooltip="Optional hard cap in milliseconds"
              className={fieldWidth}
            >
              <InputNumber
                min={0}
                max={Number.MAX_VALUE}
                step={100}
                className="w-full"
                addonAfter="ms"
              />
            </Form.Item>

            <Divider className="my-2" />

            <Typography.Text className="block mb-2">Valid Return Codes</Typography.Text>
            <Form.List name={['task_spec', 'valid_return_codes']}>
              {(fields, { add, remove }) => (
                <Space direction="vertical" className="w-full">
                  {fields.map((field) => (
                    <div key={field.key} className="w-full">
                      {isSm ? (
                        <Space.Compact className="w-full">
                          <Form.Item name={[field.name]} noStyle rules={[{ required: true }]}>
                            <InputNumber className="w-full" placeholder="Code" />
                          </Form.Item>
                          <Button
                            icon={<DeleteOutlined />}
                            onClick={() => remove(field.name)}
                            danger
                          />
                        </Space.Compact>
                      ) : (
                        <Space direction="vertical" className="w-full">
                          <Form.Item
                            name={[field.name]}
                            rules={[{ required: true }]}
                            className="!mb-0"
                          >
                            <InputNumber className="w-full" placeholder="Code" />
                          </Form.Item>
                          <div>
                            <Button
                              icon={<DeleteOutlined />}
                              onClick={() => remove(field.name)}
                              danger
                            />
                          </div>
                        </Space>
                      )}
                    </div>
                  ))}
                  <Button onClick={() => add()} icon={<PlusOutlined />}>
                    Add Return Code
                  </Button>
                </Space>
              )}
            </Form.List>

            <Divider className="my-2" />

            <Typography.Text className="block mb-2">Forbidden Outputs</Typography.Text>
            <Form.List name={['task_spec', 'forbidden_outputs']}>
              {(fields, { add, remove }) => (
                <Space direction="vertical" className="w-full">
                  {fields.map((field) => (
                    <div key={field.key} className="w-full">
                      {isSm ? (
                        <Space.Compact className="w-full">
                          <Form.Item name={[field.name]} noStyle rules={[{ required: true }]}>
                            <Input
                              className="w-full"
                              placeholder="Substring to disallow in output"
                            />
                          </Form.Item>
                          <Button
                            icon={<DeleteOutlined />}
                            onClick={() => remove(field.name)}
                            danger
                          />
                        </Space.Compact>
                      ) : (
                        <Space direction="vertical" className="w-full">
                          <Form.Item
                            name={[field.name]}
                            rules={[{ required: true }]}
                            className="!mb-0"
                          >
                            <Input
                              className="w-full"
                              placeholder="Substring to disallow in output"
                            />
                          </Form.Item>
                          <div>
                            <Button
                              icon={<DeleteOutlined />}
                              onClick={() => remove(field.name)}
                              danger
                            />
                          </div>
                        </Space>
                      )}
                    </div>
                  ))}
                  <Button onClick={() => add()} icon={<PlusOutlined />}>
                    Add Forbidden Output
                  </Button>
                </Space>
              )}
            </Form.List>
          </SettingsGroup>

          {/* ---- Runtime Flags ---- */}
          <SettingsGroup title="Runtime" description="Parallelism and verbosity for GA execution.">
            <Form.Item
              name="max_parallel_chromosomes"
              label="Max Parallel Chromosomes"
              className={fieldWidth}
              rules={[{ required: true }]}
            >
              <InputNumber min={1} step={1} precision={0} className="w-full" />
            </Form.Item>

            <Form.Item
              name="verbose"
              label="Verbose"
              valuePropName="checked"
              className={fieldWidth}
            >
              <Switch />
            </Form.Item>

            <div className="pt-2">
              <AssignmentConfigActions
                primaryText="Save GATLAM Config"
                onPrimary={onSave}
                disabled={disabled}
              />
            </div>
          </SettingsGroup>
        </Space>
      </Form>
    </div>
  );
}
