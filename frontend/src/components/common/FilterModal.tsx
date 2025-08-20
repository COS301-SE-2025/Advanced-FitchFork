import React, { useMemo, useState } from 'react';
import { Modal, Collapse, Select, Input, Space, Button, Tooltip } from 'antd';
import { ClearOutlined } from '@ant-design/icons';
import { useUI } from '@/context/UIContext';

type FilterType = 'select' | 'text' | 'number' | 'multi-select';

export interface FilterGroup {
  key: string;
  label: string;
  type: FilterType;
  options?: { label: string; value: string }[];
}

export interface FilterModalProps {
  open: boolean;
  onClose: () => void;
  filterGroups: FilterGroup[];
  /** e.g. ["status:open", "q:abc", "tag:ui"] */
  activeFilters: string[];
  onChange: (values: string[]) => void;
}

function parse(active: string[]) {
  const map = new Map<string, string[]>();
  for (const item of active || []) {
    const idx = item.indexOf(':');
    const k = idx >= 0 ? item.slice(0, idx) : item;
    const v = idx >= 0 ? item.slice(idx + 1) : '';
    if (!map.has(k)) map.set(k, []);
    if (v) map.get(k)!.push(v);
  }
  return map;
}

function format(map: Map<string, string[]>) {
  const result: string[] = [];
  map.forEach((vals, k) => {
    vals.filter(Boolean).forEach((v) => result.push(`${k}:${v}`));
  });
  return result;
}

const FilterModal: React.FC<FilterModalProps> = ({
  open,
  onClose,
  filterGroups,
  activeFilters,
  onChange,
}) => {
  const [draft, setDraft] = useState(() => parse(activeFilters));
  const { isMobile } = useUI();
  const controlSize = isMobile ? 'middle' : 'middle';

  // stable key to avoid changing deps size
  const groupsKey = useMemo(() => {
    const parts: string[] = [];
    for (const g of filterGroups) {
      parts.push(g.key, g.type);
      if (g.options?.length) parts.push(...g.options.map((o) => `${o.label}=${o.value}`));
    }
    return parts.join('|');
  }, [filterGroups]);

  React.useEffect(() => {
    if (!open) return;
    // drop values no longer available for select/multi-select
    const allowedValuesByKey = new Map<string, Set<string>>();
    for (const g of filterGroups) {
      if (g.options && (g.type === 'select' || g.type === 'multi-select')) {
        allowedValuesByKey.set(g.key, new Set(g.options.map((o) => o.value)));
      }
    }
    const next = parse(activeFilters);
    for (const [k, vals] of next) {
      const allow = allowedValuesByKey.get(k);
      if (allow)
        next.set(
          k,
          vals.filter((v) => allow.has(v)),
        );
    }
    setDraft(next);
  }, [open, activeFilters, groupsKey]);

  function setGroupValues(key: string, values: string[] | string) {
    setDraft((prev) => {
      const next = new Map(prev);
      const arr = Array.isArray(values) ? values : values ? [values] : [];
      next.set(key, arr);
      return next;
    });
  }

  function clearGroup(key: string) {
    setGroupValues(key, []);
  }

  function apply() {
    onChange(format(draft));
    onClose();
  }

  return (
    <Modal open={open} title="Filters" onCancel={onClose} footer={null} centered destroyOnClose>
      <Collapse
        bordered={false}
        items={filterGroups.map((g) => {
          const values = draft.get(g.key) || [];
          const showClear = values.length > 0;

          return {
            key: g.key,
            label: (
              <div className="flex items-center justify-between w-full">
                <span>{g.label}</span>
                {showClear && (
                  <Space.Compact>
                    <Tooltip title="Clear this filter">
                      <Button
                        size="small"
                        icon={<ClearOutlined />}
                        onClick={(e) => {
                          e.stopPropagation();
                          clearGroup(g.key);
                        }}
                      />
                    </Tooltip>
                  </Space.Compact>
                )}
              </div>
            ),
            children: (
              <div className={isMobile ? 'flex flex-col gap-2' : 'flex items-center gap-2'}>
                {g.type === 'select' && g.options && (
                  <Select
                    size={controlSize}
                    className={isMobile ? 'w-full' : 'min-w-0 flex-1'}
                    placeholder={`Select ${g.label}`}
                    value={values[0]}
                    options={g.options}
                    allowClear
                    onChange={(v) => setGroupValues(g.key, v ?? '')}
                  />
                )}

                {g.type === 'multi-select' && g.options && (
                  <Select
                    size={controlSize}
                    mode="multiple"
                    className={isMobile ? 'w-full' : 'min-w-0 flex-1'}
                    placeholder={`Select ${g.label}`}
                    value={values}
                    options={g.options}
                    onChange={(vals) => setGroupValues(g.key, vals)}
                    allowClear
                    maxTagCount={isMobile ? 'responsive' : undefined}
                  />
                )}

                {g.type === 'text' && (
                  <Input
                    size={controlSize}
                    placeholder={`Enter ${g.label}`}
                    value={values[0] ?? ''}
                    onChange={(e) => setGroupValues(g.key, e.target.value)}
                    allowClear
                    className={isMobile ? 'w-full' : 'min-w-0 flex-1'}
                  />
                )}

                {g.type === 'number' && (
                  <Input
                    size={controlSize}
                    type="number"
                    placeholder={`Enter ${g.label}`}
                    value={values[0] ?? ''}
                    onChange={(e) => setGroupValues(g.key, e.target.value)}
                    allowClear
                    className={isMobile ? 'w-full' : 'min-w-0 flex-1'}
                  />
                )}
              </div>
            ),
          };
        })}
      />

      <div className="mt-4 flex flex-col sm:flex-row gap-2 sm:items-center sm:justify-end">
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

export default FilterModal;
