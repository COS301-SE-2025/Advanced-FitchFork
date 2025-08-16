import { useEffect } from 'react';
import { Modal, Input, Select, DatePicker, Button, Checkbox, Form, Typography } from 'antd';
import dayjs from 'dayjs';

export interface TableCreateModalField {
  name: string;
  label: string;
  type: 'text' | 'textarea' | 'number' | 'email' | 'password' | 'select' | 'datetime' | 'boolean';
  placeholder?: string;
  required?: boolean;
  options?: { label: string; value: string }[];
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
}

const CreateModal = ({
  open,
  onCancel,
  onCreate,
  onChange,
  fields,
  initialValues,
  title = 'Create Item',
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
      const values = await form.validateFields();
      onCreate(values);
    } catch {
      // validation failed
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
        {fields.map(({ name, label, type, placeholder, options, required }) => {
          const rules = required ? [{ required: true, message: `Please enter ${label}` }] : [];

          if (type === 'boolean') {
            return (
              <Form.Item key={name} name={name} label={label} valuePropName="checked" rules={rules}>
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
                <Select options={options} placeholder={placeholder} />
              ) : type === 'datetime' ? (
                <DatePicker
                  showTime={{ format: 'HH:mm' }}
                  format="YYYY-MM-DD HH:mm"
                  style={{ width: '100%' }}
                />
              ) : null}
            </Form.Item>
          );
        })}

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
