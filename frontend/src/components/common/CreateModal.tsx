// components/common/CreateModal.tsx
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
} from 'antd';
import type { SelectProps, TreeSelectProps } from 'antd';
import dayjs from 'dayjs';

export interface TableCreateModalField {
  name: string;
  label: string;
  type:
    | 'text'
    | 'textarea'
    | 'number'
    | 'email'
    | 'password'
    | 'select'
    | 'tree-select'
    | 'datetime'
    | 'boolean';
  placeholder?: string;
  required?: boolean;

  /** For Select: ReactNode labels + string/number values */
  options?: { label: React.ReactNode; value: string | number; disabled?: boolean }[];

  /** For TreeSelect: treeData (reuse `options` field) */
  treeData?: {
    title: React.ReactNode;
    value: string | number;
    disabled?: boolean;
    selectable?: boolean;
    children?: any[];
  }[];

  /** Forward arbitrary AntD props */
  selectProps?: SelectProps;
  treeSelectProps?: TreeSelectProps;

  defaultValue?: any;
}

interface Props {
  open: boolean;
  onCancel: () => void;
  onCreate: (values: Record<string, any>) => void;
  onChange?: (values: Record<string, any>) => void;
  fields: TableCreateModalField[];
  initialValues: Record<string, any>;
  title?: string;
  normalizeDatetime?: boolean; // default false
}

const CreateModal = ({
  open,
  onCancel,
  onCreate,
  onChange,
  fields,
  initialValues,
  title = 'Create Item',
  normalizeDatetime = false,
}: Props) => {
  const [form] = Form.useForm();

  useEffect(() => {
    if (open) {
      const values = { ...initialValues };
      fields.forEach((field) => {
        if (values[field.name] === undefined && field.defaultValue !== undefined) {
          values[field.name] = field.defaultValue;
        }
        if (field.type === 'datetime' && values[field.name]) {
          values[field.name] = dayjs(values[field.name]);
        }
      });
      form.setFieldsValue(values);
    }
  }, [open, initialValues, fields, form]);

  const handleValuesChange = (_: any, allValues: any) => onChange?.(allValues);

  const handleSubmit = async () => {
    try {
      const raw = await form.validateFields();
      const values = { ...raw };

      // keep numbers numeric
      fields.forEach((f) => {
        if (f.type === 'number' && typeof values[f.name] === 'string') {
          const n = Number(values[f.name]);
          if (!Number.isNaN(n)) values[f.name] = n;
        }
      });

      // optional normalize datetimes
      if (normalizeDatetime) {
        fields.forEach((f) => {
          const v = values[f.name];
          if (f.type === 'datetime' && v && typeof v?.isValid === 'function' && v.isValid()) {
            values[f.name] = v.toISOString();
          }
        });
      }

      onCreate(values);
    } catch {
      /* validation errors already shown */
    }
  };

  return (
    <Modal
      open={open}
      onCancel={onCancel}
      footer={null}
      title={<Typography.Title level={4}>{title}</Typography.Title>}
      centered
      data-testid="create-modal"
    >
      <Form layout="vertical" form={form} onValuesChange={handleValuesChange} className="space-y-4">
        {fields.map(
          ({
            name,
            label,
            type,
            placeholder,
            options,
            treeData,
            required,
            selectProps,
            treeSelectProps,
          }) => {
            const rules = required ? [{ required: true, message: `Please enter ${label}` }] : [];

            if (type === 'boolean') {
              return (
                <Form.Item
                  key={name}
                  name={name}
                  label={label}
                  valuePropName="checked"
                  rules={rules}
                >
                  <Checkbox>{placeholder || 'Yes'}</Checkbox>
                </Form.Item>
              );
            }

            return (
              <Form.Item key={name} name={name} label={label} rules={rules}>
                {type === 'password' ? (
                  <Input.Password placeholder={placeholder} />
                ) : type === 'text' || type === 'email' || type === 'number' ? (
                  <Input type={type === 'number' ? 'number' : type} placeholder={placeholder} />
                ) : type === 'textarea' ? (
                  <Input.TextArea rows={4} placeholder={placeholder} />
                ) : type === 'select' ? (
                  <Select
                    options={options}
                    placeholder={placeholder}
                    optionLabelProp="label"
                    {...selectProps}
                  />
                ) : type === 'tree-select' ? (
                  <TreeSelect
                    treeData={treeData ?? (options as any)}
                    treeNodeLabelProp="title" // show node title when selected
                    showSearch
                    placeholder={placeholder}
                    style={{ width: '100%' }}
                    {...treeSelectProps}
                  />
                ) : type === 'datetime' ? (
                  <DatePicker
                    showTime={{ format: 'HH:mm' }}
                    format="YYYY-MM-DD HH:mm"
                    style={{ width: '100%' }}
                  />
                ) : null}
              </Form.Item>
            );
          },
        )}

        <Form.Item>
          <div className="flex justify-end gap-2 pt-2">
            <Button onClick={onCancel} data-testid="create-modal-cancel">
              Cancel
            </Button>
            <Button type="primary" onClick={handleSubmit} data-testid="create-modal-submit">
              Create
            </Button>
          </div>
        </Form.Item>
      </Form>
    </Modal>
  );
};

export default CreateModal;
