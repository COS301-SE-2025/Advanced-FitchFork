import React, { useMemo, useState } from 'react';
import { Modal, Select, Segmented, Button, Tooltip, Space } from 'antd';
import {
  ArrowUpOutlined,
  ArrowDownOutlined,
  PlusOutlined,
  DeleteOutlined,
} from '@ant-design/icons';

export interface SortOption {
  label: string;
  field: string;
}

type Order = 'ascend' | 'descend';

export interface SortModalProps {
  open: boolean;
  onClose: () => void;
  sortOptions: SortOption[];
  /** e.g. ["name.ascend", "createdAt.descend"] */
  currentSort: string[];
  onChange: (value: string[]) => void;
}

const orderOptions: { label: React.ReactNode; value: Order }[] = [
  { label: 'Asc', value: 'ascend' },
  { label: 'Desc', value: 'descend' },
];

function parseSort(current: string[]) {
  const seen = new Set<string>();
  const result: { field: string; order: Order }[] = [];
  for (const s of current || []) {
    const [field, orderRaw] = s.split('.');
    if (!field || seen.has(field)) continue;
    const order: Order = orderRaw === 'descend' ? 'descend' : 'ascend';
    seen.add(field);
    result.push({ field, order });
  }
  return result;
}

function formatSort(items: { field: string; order: Order }[]) {
  return items.filter((it) => it.field).map((it) => `${it.field}.${it.order}`);
}

const SortModal: React.FC<SortModalProps> = ({
  open,
  onClose,
  sortOptions,
  currentSort,
  onChange,
}) => {
  const [draft, setDraft] = useState(() => parseSort(currentSort));

  const fieldOptions = useMemo(
    () => sortOptions.map((o) => ({ label: o.label, value: o.field })),
    [sortOptions],
  );
  const fieldsKey = useMemo(() => fieldOptions.map((o) => o.value).join('|'), [fieldOptions]);

  // re-sync draft when opening or when options change
  React.useEffect(() => {
    if (!open) return;
    const allowed = new Set(fieldOptions.map((o) => o.value));
    const normalized = parseSort(currentSort ?? []).filter((r) => allowed.has(r.field));
    setDraft(normalized);
  }, [open, currentSort, fieldsKey]);

  function updateItem(idx: number, patch: Partial<{ field: string; order: Order }>) {
    setDraft((prev) => {
      const next = [...prev];
      const updated = { ...next[idx], ...patch };
      if (patch.field) {
        const dup = next.some((r, i) => i !== idx && r.field === patch.field);
        if (dup) return prev; // block duplicates
      }
      next[idx] = updated;
      return next;
    });
  }

  function addLevel() {
    const used = new Set(draft.map((d) => d.field));
    const candidate = fieldOptions.find((o) => !used.has(o.value))?.value;
    if (!candidate) return;
    setDraft((prev) => [...prev, { field: candidate, order: 'ascend' as Order }]);
  }

  function removeLevel(idx: number) {
    setDraft((prev) => prev.filter((_, i) => i !== idx));
  }

  function move(idx: number, dir: -1 | 1) {
    setDraft((prev) => {
      const next = [...prev];
      const target = idx + dir;
      if (target < 0 || target >= next.length) return prev;
      [next[idx], next[target]] = [next[target], next[idx]];
      return next;
    });
  }

  function apply() {
    onChange(formatSort(draft));
    onClose();
  }

  const controlSize: 'middle' = 'middle';
  const maxReached = draft.length >= fieldOptions.length;

  return (
    <Modal open={open} title="Sort" onCancel={onClose} footer={null} centered destroyOnClose>
      <div className="space-y-3">
        {/* Empty state */}
        {draft.length === 0 && (
          <div className="text-gray-500 text-center py-4">No sort levels added yet.</div>
        )}

        {draft.map((row, idx) => {
          const isFirst = idx === 0;
          const isLast = idx === draft.length - 1;

          const usedInOthers = new Set(draft.filter((_, i) => i !== idx).map((r) => r.field));
          const perRowOptions = fieldOptions.map((o) => ({
            ...o,
            disabled: usedInOthers.has(o.value),
          }));

          return (
            <div key={idx} className="flex items-stretch gap-3">
              {/* Inline controls like desktop */}
              <div className="flex-1 min-w-0 flex items-center gap-2">
                <Select
                  size={controlSize}
                  value={row.field}
                  options={perRowOptions}
                  onChange={(v) => updateItem(idx, { field: v })}
                  className="min-w-0 flex-1"
                  optionLabelProp="label"
                  popupMatchSelectWidth={false}
                />
                <Segmented
                  size={controlSize as any}
                  value={row.order}
                  onChange={(v) => updateItem(idx, { order: v as Order })}
                  options={orderOptions}
                />
              </div>

              {/* Actions */}
              <Space.Compact>
                <Tooltip title="Move up">
                  <Button
                    size={controlSize}
                    icon={<ArrowUpOutlined />}
                    onClick={() => move(idx, -1)}
                    disabled={isFirst}
                  />
                </Tooltip>
                <Tooltip title="Move down">
                  <Button
                    size={controlSize}
                    icon={<ArrowDownOutlined />}
                    onClick={() => move(idx, +1)}
                    disabled={isLast}
                  />
                </Tooltip>
                <Tooltip title="Remove">
                  <Button
                    size={controlSize}
                    icon={<DeleteOutlined />}
                    danger
                    onClick={() => removeLevel(idx)}
                  />
                </Tooltip>
              </Space.Compact>
            </div>
          );
        })}
      </div>

      {/* Add level below the list with helpful tooltip when disabled */}
      <div className="mt-4 flex flex-col sm:flex-row gap-2 sm:items-center sm:justify-between">
        <Tooltip
          title={
            fieldOptions.length === 0
              ? 'No fields available to sort by'
              : maxReached
                ? 'All fields are already used'
                : ''
          }
        >
          <Button
            size={controlSize}
            icon={<PlusOutlined />}
            onClick={addLevel}
            className="w-full sm:w-auto"
            disabled={fieldOptions.length === 0 || maxReached}
          >
            Add level
          </Button>
        </Tooltip>

        <div className="flex gap-2">
          <Button size={controlSize} onClick={onClose} className="w-full sm:w-auto">
            Close
          </Button>
          <Button size={controlSize} type="primary" onClick={apply} className="w-full sm:w-auto">
            Apply
          </Button>
        </div>
      </div>
    </Modal>
  );
};

export default SortModal;
