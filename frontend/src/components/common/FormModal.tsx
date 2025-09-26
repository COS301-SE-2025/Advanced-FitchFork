// components/common/FormModal.tsx
import React, { useEffect } from 'react';
import {
  Modal,
  Input,
  Select,
  DatePicker,
  Button,
  Checkbox,
  Form,
  Typography,
  TreeSelect,
  InputNumber,
} from 'antd';
import type { Rule } from 'antd/es/form';
import type { SelectProps, TreeSelectProps, InputNumberProps } from 'antd';
import dayjs, { Dayjs } from 'dayjs';

type FieldType =
  | 'text'
  | 'textarea'
  | 'number'
  | 'email'
  | 'password'
  | 'select'
  | 'tree-select'
  | 'datetime'
  | 'boolean';

/** Single source of truth for both UI and validation */
type Constraints =
  | undefined
  | {
      required?: boolean | string; // string = message
      email?: { message?: string }; // usually inferred by type:'email'
      length?: { min?: number; max?: number; messageMin?: string; messageMax?: string };
      pattern?: { regex: RegExp; message?: string };
      number?: {
        min?: number;
        max?: number;
        integer?: boolean;
        step?: number; // also applied to UI
        precision?: number; // UI + validator will allow decimals unless integer
        message?: string;
        messageMin?: string;
        messageMax?: string;
      };
      date?: {
        min?: string | Dayjs;
        max?: string | Dayjs;
        messageMin?: string;
        messageMax?: string;
        // UI hints; will default sensible values if omitted
        format?: string;
        withTime?: boolean;
      };
      custom?: {
        validator: (value: any, allValues: any) => Promise<void> | void;
        message?: string; // optional default message
      };
    };

export interface FormModalField {
  name: string;
  label: string;
  type: FieldType;
  placeholder?: string;

  /** Provide choices for select/tree-select */
  options?: { label: React.ReactNode; value: string | number; disabled?: boolean }[];
  treeData?: {
    title: React.ReactNode;
    value: string | number;
    disabled?: boolean;
    selectable?: boolean;
    children?: any[];
  }[];

  /** Declare constraints ONCE; they power UI + validation */
  constraints?: Constraints;

  /** Rare UI overrides (avoid unless necessary) */
  ui?: { props?: Record<string, any> };

  defaultValue?: any;
}

interface Props {
  open: boolean;
  title?: string;
  submitText?: string;
  cancelText?: string;
  initialValues: Record<string, any>;
  fields: FormModalField[];
  onSubmit: (values: Record<string, any>) => void | Promise<void>;
  onCancel: () => void;
  onChange?: (values: Record<string, any>) => void;
  normalizeDatetime?: boolean; // to ISO on submit
  modalTestId?: string;
  submitTestId?: string;
  cancelTestId?: string;
}

const toDayjs = (v: any) => (dayjs.isDayjs(v) ? v : dayjs(v));

/** Derive AntD Form rules from constraints (no duplication) */
const rulesFromConstraints = (
  f: FormModalField,
  form: ReturnType<typeof Form.useForm>[0],
): Rule[] => {
  const c = f.constraints;
  if (!c) return [];

  const rs: Rule[] = [];

  if (c.required) {
    rs.push({
      required: true,
      message: typeof c.required === 'string' ? c.required : `Please enter ${f.label}`,
    });
  }

  if (f.type === 'email' || c.email) {
    rs.push({ type: 'email', message: c.email?.message ?? 'Please enter a valid email address' });
  }

  if (c.length) {
    const { min, max, messageMin, messageMax } = c.length;
    if (typeof min === 'number')
      rs.push({ min, message: messageMin ?? `Minimum ${min} characters` });
    if (typeof max === 'number')
      rs.push({ max, message: messageMax ?? `Maximum ${max} characters` });
  }

  if (c.pattern) {
    rs.push({ pattern: c.pattern.regex, message: c.pattern.message ?? `Invalid ${f.label}` });
  }

  if (c.number && f.type === 'number') {
    const { min, max, integer, message, messageMin, messageMax } = c.number;
    rs.push({
      validator: async (_rule, value) => {
        const isReq = !!c.required;
        if (value === undefined || value === null || value === '') {
          if (isReq) throw new Error(`Please enter ${f.label}`);
          return;
        }
        if (typeof value !== 'number' || Number.isNaN(value))
          throw new Error(message ?? 'Must be a number');
        if (integer && !Number.isInteger(value)) throw new Error('Must be an integer');
        if (typeof min === 'number' && value < min)
          throw new Error(messageMin ?? `Must be ≥ ${min}`);
        if (typeof max === 'number' && value > max)
          throw new Error(messageMax ?? `Must be ≤ ${max}`);
      },
    });
  }

  if (c.date && f.type === 'datetime') {
    const { min, max, messageMin, messageMax } = c.date;
    rs.push({
      validator: async (_rule, value: Dayjs | null) => {
        const isReq = !!c.required;
        if (!value) {
          if (isReq) throw new Error(`Please select ${f.label}`);
          return;
        }
        const d = toDayjs(value);
        if (!d.isValid()) throw new Error('Please select a valid date/time');
        if (min) {
          const md = toDayjs(min);
          if (d.isBefore(md))
            throw new Error(messageMin ?? `Must be on/after ${md.format('YYYY-MM-DD HH:mm')}`);
        }
        if (max) {
          const xd = toDayjs(max);
          if (d.isAfter(xd))
            throw new Error(messageMax ?? `Must be on/before ${xd.format('YYYY-MM-DD HH:mm')}`);
        }
      },
    });
  }

  if (c.custom) {
    rs.push({
      validator: async () => {
        const all = form.getFieldsValue(true);
        await c.custom!.validator(all[f.name], all);
      },
      message: c.custom.message,
    } as Rule);
  }

  return rs;
};

/** Derive control props (UI) from constraints (no duplication) */
const controlPropsFromConstraints = (f: FormModalField): Record<string, any> => {
  const c = f.constraints;
  const overrides = f.ui?.props ?? {};

  if (!c) return overrides; // only overrides provided

  switch (f.type) {
    case 'number': {
      const p: InputNumberProps = {};
      if (c.number?.min !== undefined) p.min = c.number.min;
      if (c.number?.max !== undefined) p.max = c.number.max;
      if (c.number?.step !== undefined) p.step = c.number.step;
      if (c.number?.precision !== undefined) p.precision = c.number.precision;
      // full-width sensible default
      p.style = { width: '100%', ...(overrides.style || {}) };
      return { ...p, ...overrides };
    }

    case 'datetime': {
      const fmt =
        c.date?.format ?? ((c.date?.withTime ?? true) ? 'YYYY-MM-DD HH:mm' : 'YYYY-MM-DD');
      const showTime =
        (c.date?.withTime ?? true) ? { format: fmt.includes('HH') ? 'HH:mm' : 'HH:mm' } : undefined;
      return {
        format: fmt,
        showTime,
        style: { width: '100%', ...(overrides.style || {}) },
        ...overrides,
      };
    }

    case 'select': {
      const sp: SelectProps = {
        options: f.options,
        placeholder: f.placeholder,
        optionLabelProp: 'label',
      };
      return { ...sp, ...overrides };
    }

    case 'tree-select': {
      const tp: TreeSelectProps = {
        treeData: f.treeData ?? (f.options as any),
        showSearch: true,
        placeholder: f.placeholder,
        style: { width: '100%' },
        treeNodeLabelProp: 'title',
      };
      // Merge style last to allow override.style to win
      return { ...tp, style: { ...tp.style, ...(overrides.style || {}) }, ...overrides };
    }

    case 'text':
    case 'email':
    case 'password': {
      return { placeholder: f.placeholder, ...overrides };
    }

    case 'textarea': {
      const rows = 4;
      return { placeholder: f.placeholder, rows, ...overrides };
    }

    case 'boolean': {
      return { ...overrides };
    }

    default:
      return overrides;
  }
};

const FormModal = ({
  open,
  title = 'Edit',
  submitText = 'Save',
  cancelText = 'Cancel',
  initialValues,
  fields,
  onSubmit,
  onCancel,
  onChange,
  normalizeDatetime = false,
  modalTestId = 'form-modal',
  submitTestId = 'form-modal-submit',
  cancelTestId = 'form-modal-cancel',
}: Props) => {
  const [form] = Form.useForm();

  useEffect(() => {
    if (!open) return;
    const values = { ...initialValues };
    fields.forEach((field) => {
      if (values[field.name] === undefined && field.defaultValue !== undefined) {
        values[field.name] = field.defaultValue;
      }
      if (field.type === 'datetime' && values[field.name]) {
        values[field.name] = toDayjs(values[field.name]);
      }
    });
    form.setFieldsValue(values);
  }, [open, initialValues, fields, form]);

  const handleValuesChange = (_: any, allValues: any) => onChange?.(allValues);

  const handleSubmit = async () => {
    try {
      const raw = await form.validateFields();
      const values = { ...raw };

      // Coerce numeric strings just in case
      fields.forEach((f) => {
        if (f.type === 'number') {
          const v = values[f.name];
          if (typeof v === 'string') {
            const n = Number(v);
            if (!Number.isNaN(n)) values[f.name] = n;
          }
        }
      });

      if (normalizeDatetime) {
        fields.forEach((f) => {
          const v = values[f.name];
          if (f.type === 'datetime' && v && typeof v?.isValid === 'function' && v.isValid()) {
            values[f.name] = (v as Dayjs).toISOString();
          }
        });
      }

      await onSubmit(values);
    } catch {
      // AntD will show validation errors
    }
  };

  const renderControl = (f: FormModalField) => {
    const cp = controlPropsFromConstraints(f);

    switch (f.type) {
      case 'password':
        return <Input.Password {...cp} />;
      case 'email':
      case 'text':
        return <Input type={f.type === 'email' ? 'email' : 'text'} {...cp} />;
      case 'textarea':
        return <Input.TextArea {...cp} />;
      case 'number':
        return <InputNumber {...(cp as InputNumberProps)} />;
      case 'select':
        return <Select {...(cp as SelectProps)} />;
      case 'tree-select':
        return <TreeSelect {...cp} />;
      case 'datetime':
        return <DatePicker {...cp} />;
      case 'boolean':
        return <Checkbox {...cp}>{f.placeholder ?? 'Yes'}</Checkbox>;
      default:
        return null;
    }
  };

  return (
    <Modal
      open={open}
      onCancel={onCancel}
      footer={null}
      title={
        <Typography.Title level={4} className="!mb-0">
          {title}
        </Typography.Title>
      }
      centered
      data-testid={modalTestId}
      rootClassName="
        dark:[&_.ant-modal-content]:!bg-gray-900 
        dark:[&_.ant-modal-content]:!text-gray-100 
        [&_.ant-modal-header]:!bg-transparent
        [&_.ant-modal-header]:!border-b-0
      "
    >
      <Form layout="vertical" form={form} onValuesChange={handleValuesChange} className="space-y-4">
        {fields.map((f) => {
          const itemRules = rulesFromConstraints(f, form);

          if (f.type === 'boolean') {
            return (
              <Form.Item
                key={f.name}
                name={f.name}
                label={f.label}
                valuePropName="checked"
                rules={itemRules}
              >
                {renderControl(f)}
              </Form.Item>
            );
          }

          return (
            <Form.Item key={f.name} name={f.name} label={f.label} rules={itemRules}>
              {renderControl(f)}
            </Form.Item>
          );
        })}

        <Form.Item>
          <div className="flex justify-end gap-2 pt-2">
            <Button onClick={onCancel} data-testid={cancelTestId}>
              {cancelText}
            </Button>
            <Button type="primary" onClick={handleSubmit} data-testid={submitTestId}>
              {submitText}
            </Button>
          </div>
        </Form.Item>
      </Form>
    </Modal>
  );
};

export default FormModal;
