import { useEffect } from 'react';
import { Typography, Form, InputNumber, Select, Switch, Button, Space, Divider, Input } from 'antd';
import { PlusOutlined, DeleteOutlined } from '@ant-design/icons';

import SettingsGroup from '@/components/SettingsGroup';
import { ConfigActions } from '@/context/AssignmentConfigContext';
import { useViewSlot } from '@/context/ViewSlotContext';

import { CROSSOVER_TYPE_OPTIONS, MUTATION_TYPE_OPTIONS } from '@/types/modules/assignments/config';

export default function GatlamPage() {
  const { setValue } = useViewSlot();

  useEffect(() => {
    setValue(
      <Typography.Text className="text-base font-medium text-gray-900 dark:text-gray-100 truncate">
        GA / TLAM Configuration
      </Typography.Text>,
    );
  }, []);

  const fieldWidth = 'w-full max-w-xs';

  return (
    <div className="flex flex-col gap-4">
      {/* ---- GA Core ---- */}
      <SettingsGroup
        title="Genetic Algorithm"
        description="Control population dynamics, selection, and genetic operators."
      >
        <Form.Item
          name={['gatlam', 'population_size']}
          label="Population Size"
          className={fieldWidth}
        >
          <InputNumber min={1} step={1} precision={0} className="w-full" />
        </Form.Item>

        <Form.Item
          name={['gatlam', 'number_of_generations']}
          label="Generations"
          className={fieldWidth}
        >
          <InputNumber min={1} step={1} precision={0} className="w-full" />
        </Form.Item>

        <Form.Item
          name={['gatlam', 'selection_size']}
          label="Selection Size"
          className={fieldWidth}
        >
          <InputNumber min={1} step={1} precision={0} className="w-full" />
        </Form.Item>

        <Form.Item
          name={['gatlam', 'reproduction_probability']}
          label="Reproduction Prob."
          className={fieldWidth}
        >
          <InputNumber min={0} max={1} step={0.01} className="w-full" />
        </Form.Item>

        <Form.Item
          name={['gatlam', 'crossover_probability']}
          label="Crossover Prob."
          className={fieldWidth}
        >
          <InputNumber min={0} max={1} step={0.01} className="w-full" />
        </Form.Item>

        <Form.Item
          name={['gatlam', 'mutation_probability']}
          label="Mutation Prob."
          className={fieldWidth}
        >
          <InputNumber min={0} max={1} step={0.001} className="w-full" />
        </Form.Item>

        <Form.Item
          name={['gatlam', 'crossover_type']}
          label="Crossover Type"
          className={fieldWidth}
        >
          <Select className="w-full" options={CROSSOVER_TYPE_OPTIONS} />
        </Form.Item>

        <Form.Item name={['gatlam', 'mutation_type']} label="Mutation Type" className={fieldWidth}>
          <Select className="w-full" options={MUTATION_TYPE_OPTIONS} />
        </Form.Item>
      </SettingsGroup>

      {/* ---- Genes ---- */}
      <SettingsGroup
        title="Genes"
        description="Define the value ranges for each gene in a chromosome."
      >
        <Form.List name={['gatlam', 'genes']}>
          {(fields, { add, remove }) => (
            <Space direction="vertical" className="w-full">
              {fields.map((field) => (
                <Space.Compact key={field.key} className="w-full">
                  <Form.Item {...field} name={[field.name, 'min_value']} noStyle>
                    <InputNumber className="w-full" placeholder="Min" />
                  </Form.Item>
                  <Form.Item {...field} name={[field.name, 'max_value']} noStyle>
                    <InputNumber className="w-full" placeholder="Max" />
                  </Form.Item>
                  <Button icon={<DeleteOutlined />} onClick={() => remove(field.name)} danger />
                </Space.Compact>
              ))}
              <Button onClick={() => add()} icon={<PlusOutlined />}>
                Add Gene
              </Button>
            </Space>
          )}
        </Form.List>
      </SettingsGroup>

      {/* ---- Component Weights ---- */}
      <SettingsGroup
        title="Component Weights"
        description="Tune the weighted components used in the GA objective."
      >
        <Form.Item name={['gatlam', 'omega1']} label="ω₁" className={fieldWidth}>
          <InputNumber step={0.01} className="w-full" />
        </Form.Item>
        <Form.Item name={['gatlam', 'omega2']} label="ω₂" className={fieldWidth}>
          <InputNumber step={0.01} className="w-full" />
        </Form.Item>
        <Form.Item name={['gatlam', 'omega3']} label="ω₃" className={fieldWidth}>
          <InputNumber step={0.01} className="w-full" />
        </Form.Item>
      </SettingsGroup>

      {/* ---- TaskSpec ---- */}
      <SettingsGroup
        title="Task Specification"
        description="Runtime and validation rules for executing chromosomes."
      >
        <Form.Item
          name={['gatlam', 'task_spec', 'max_runtime_ms']}
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
        <Form.List name={['gatlam', 'task_spec', 'valid_return_codes']}>
          {(fields, { add, remove }) => (
            <Space direction="vertical" className="w-full">
              {fields.map((field) => (
                <Space.Compact key={field.key} className="w-full">
                  <Form.Item {...field} name={[field.name]} noStyle>
                    <InputNumber className="w-full" placeholder="Code" />
                  </Form.Item>
                  <Button icon={<DeleteOutlined />} onClick={() => remove(field.name)} danger />
                </Space.Compact>
              ))}
              <Button onClick={() => add()} icon={<PlusOutlined />}>
                Add Return Code
              </Button>
            </Space>
          )}
        </Form.List>

        <Divider className="my-2" />

        <Typography.Text className="block mb-2">Forbidden Outputs</Typography.Text>
        <Form.List name={['gatlam', 'task_spec', 'forbidden_outputs']}>
          {(fields, { add, remove }) => (
            <Space direction="vertical" className="w-full">
              {fields.map((field) => (
                <Space.Compact key={field.key} className="w-full">
                  <Form.Item {...field} name={[field.name]} noStyle className="flex-1">
                    <Input className="w-full" placeholder="Substring to disallow in output" />
                  </Form.Item>
                  <Button icon={<DeleteOutlined />} onClick={() => remove(field.name)} danger />
                </Space.Compact>
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
          name={['gatlam', 'max_parallel_chromosomes']}
          label="Max Parallel Chromosomes"
          className={fieldWidth}
        >
          <InputNumber min={1} step={1} precision={0} className="w-full" />
        </Form.Item>

        <Form.Item
          name={['gatlam', 'verbose']}
          label="Verbose"
          valuePropName="checked"
          className={fieldWidth}
        >
          <Switch />
        </Form.Item>

        <ConfigActions saveLabel="Save GA / TLAM Config" />
      </SettingsGroup>
    </div>
  );
}
