import { useEffect, useState } from 'react';
import { Modal, Input, Select, DatePicker, Button, Checkbox } from 'antd';
import dayjs from 'dayjs';

export interface TableCreateModalField {
  name: string;
  label: string;
  type: 'text' | 'textarea' | 'number' | 'email' | 'password' | 'select' | 'datetime' | 'boolean';
  placeholder?: string;
  required?: boolean;
  options?: { label: string; value: string }[]; // only for 'select'
}

export interface TableCreateModalProps {
  open: boolean;
  onCancel: () => void;
  onCreate: (values: Record<string, any>) => void;
  onChange?: (values: Record<string, any>) => void;
  fields: TableCreateModalField[];
  initialValues: Record<string, any>;
  title?: string;
}

export default function TableCreateModal({
  open,
  onCancel,
  onCreate,
  onChange,
  fields,
  initialValues,
  title = 'Create Item',
}: TableCreateModalProps) {
  const [formState, setFormState] = useState(initialValues);

  useEffect(() => {
    if (open) setFormState(initialValues);
  }, [open, initialValues]);

  const updateField = (name: string, value: any) => {
    setFormState((prev) => {
      const updated = { ...prev, [name]: value };
      onChange?.(updated);
      return updated;
    });
  };

  return (
    <Modal open={open} onCancel={onCancel} footer={null} title={title} destroyOnClose centered>
      <div className="space-y-4">
        {fields.map(({ name, label, type, options, placeholder }) => (
          <div key={name}>
            <label className="block text-sm font-medium mb-1">{label}</label>
            {type === 'text' || type === 'email' || type === 'password' || type === 'number' ? (
              <Input
                type={type === 'number' ? 'number' : type}
                placeholder={placeholder}
                value={formState[name]}
                onChange={(e) => updateField(name, e.target.value)}
              />
            ) : null}
            {type === 'textarea' && (
              <Input.TextArea
                rows={4}
                placeholder={placeholder}
                value={formState[name]}
                onChange={(e) => updateField(name, e.target.value)}
              />
            )}
            {type === 'select' && (
              <Select
                value={formState[name]}
                onChange={(e) => updateField(name, e.target.value)}
                options={options}
                style={{ width: '100%' }}
              />
            )}
            {type === 'datetime' && (
              <DatePicker
                showTime={{ format: 'HH:mm' }}
                value={formState[name] ? dayjs(formState[name]) : undefined}
                onChange={(e) => updateField(name, e?.format('YYYY-MM-DD HH:mm'))}
                format="YYYY-MM-DD HH:mm"
                style={{ width: '100%' }}
              />
            )}
            {type === 'boolean' && (
              <Checkbox
                checked={!!formState[name]}
                onChange={(e) => updateField(name, e.target.checked)}
              >
                {placeholder || 'Yes'}
              </Checkbox>
            )}
          </div>
        ))}

        <div className="flex gap-2 mt-6">
          <Button onClick={onCancel} className="w-1/2">
            Cancel
          </Button>
          <Button type="primary" onClick={() => onCreate(formState)} className="w-1/2">
            Create
          </Button>
        </div>
      </div>
    </Modal>
  );
}
