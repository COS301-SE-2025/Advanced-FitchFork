import React, { useState } from 'react';
import { Input, Button, Dropdown, Segmented, Select, Modal, Col, Row, Space, Checkbox } from 'antd';
import { ReloadOutlined, TableOutlined, AppstoreOutlined, MoreOutlined } from '@ant-design/icons';
import type { MenuItemType } from 'antd/es/menu/interface';
import type { EntityAction } from './EntityList';

const { Search } = Input;

interface SortOption {
  label: string;
  field: string;
}

type FilterType = 'select' | 'text' | 'number' | 'multi-select';

interface FilterGroup {
  key: string;
  label: string;
  type: FilterType;
  options?: { label: string; value: string }[];
}

interface Props<T> {
  handleSearch: (key: string) => void;
  searchTerm: string;
  viewMode?: 'table' | 'grid';
  onViewModeChange?: (val: 'table' | 'grid') => void;

  selectedRowKeys?: React.Key[];
  searchPlaceholder?: string;

  sortOptions?: SortOption[];
  currentSort?: string[];
  onSortChange?: (value: string[]) => void;

  filterGroups?: FilterGroup[];
  activeFilters?: string[];
  onFilterChange?: (values: string[]) => void;

  actions?: EntityAction<T>[];
  bulkActions?: EntityAction<T>[];

  columnToggleEnabled?: boolean;
  columns?: { key: string; label: string; defaultHidden?: boolean }[];
  hiddenColumns?: Set<string>;
  onToggleColumn?: (key: string) => void;
}

const ControlBar = <T,>({
  handleSearch,
  searchTerm,
  viewMode,
  onViewModeChange,
  selectedRowKeys = [],
  searchPlaceholder = 'Search...',
  sortOptions = [],
  currentSort = [],
  onSortChange,
  filterGroups = [],
  activeFilters = [],
  onFilterChange,
  actions = [],
  bulkActions = [],
  columnToggleEnabled = false,
  columns = [],
  hiddenColumns = new Set(),
  onToggleColumn = () => {},
}: Props<T>) => {
  const [sortModalOpen, setSortModalOpen] = useState(false);
  const [filterModalOpen, setFilterModalOpen] = useState(false);

  const hasBulk = selectedRowKeys.length > 0 && bulkActions.length > 0;

  const hasSearch = !!searchTerm.trim();
  const hasSort = (currentSort?.length ?? 0) > 0;
  const hasFilters = (activeFilters?.length ?? 0) > 0;

  const hasActiveFilters = hasSearch || hasSort || hasFilters;

  const clearMenuItems: MenuItemType[] = [
    hasSearch && {
      key: 'clear-search',
      label: <span data-cy="clear-search">Clear Search</span>,
      onClick: () => handleSearch(''),
    },
    hasSort && {
      key: 'clear-sort',
      label: <span data-cy="clear-sort">Clear Sort</span>,
      onClick: () => onSortChange?.([]),
    },
    hasFilters && {
      key: 'clear-filters',
      label: <span data-cy="clear-filters">Clear Filters</span>,
      onClick: () => onFilterChange?.([]),
    },
  ].filter(Boolean) as MenuItemType[];

  if (clearMenuItems.length > 1) {
    clearMenuItems.push({
      key: 'clear-all',
      label: <span data-cy="clear-all">Clear All</span>,
      onClick: () => {
        handleSearch('');
        onSortChange?.([]);
        onFilterChange?.([]);
      },
    });
  }

  const primaryAction = actions.find((a) => a.isPrimary) ?? actions[0] ?? null;
  const secondaryActions = primaryAction ? actions.filter((a) => a.key !== primaryAction.key) : [];

  const resolvedPrimaryBulk = bulkActions.find((a) => a.isPrimary) ?? bulkActions[0] ?? null;
  const secondaryBulkActions = resolvedPrimaryBulk
    ? bulkActions.filter((a) => a.key !== resolvedPrimaryBulk.key)
    : [];

  const columnToggleMenu = (
    <Dropdown
      trigger={['click']}
      menu={{
        items: [
          ...columns?.map((col) => ({
            key: col.key,
            label: (
              <div onClick={(e) => e.stopPropagation()}>
                <Checkbox
                  checked={!hiddenColumns?.has(col.key)}
                  onChange={() => onToggleColumn?.(col.key)}
                >
                  {col.label}
                </Checkbox>
              </div>
            ),
          })),

          { type: 'divider' },

          {
            key: 'showAll',
            label: (
              <Button
                type="link"
                size="small"
                onClick={(e) => {
                  e.stopPropagation();
                  columns?.forEach((col) => {
                    if (hiddenColumns?.has(col.key)) {
                      onToggleColumn?.(col.key);
                    }
                  });
                }}
              >
                Show All
              </Button>
            ),
          },

          {
            key: 'hideAll',
            label: (
              <Button
                type="link"
                size="small"
                onClick={(e) => {
                  e.stopPropagation();
                  columns?.forEach((col) => {
                    if (!hiddenColumns?.has(col.key)) {
                      onToggleColumn?.(col.key);
                    }
                  });
                }}
              >
                Hide All
              </Button>
            ),
          },

          {
            key: 'resetDefault',
            label: (
              <Button
                type="link"
                size="small"
                onClick={(e) => {
                  e.stopPropagation();
                  columns?.forEach((col) => {
                    const shouldBeHidden = !!(col as any).defaultHidden;
                    const currentlyHidden = hiddenColumns?.has(col.key);

                    // if current != default, toggle
                    if (shouldBeHidden !== currentlyHidden) {
                      onToggleColumn?.(col.key);
                    }
                  });
                }}
              >
                Reset to Default
              </Button>
            ),
          },
        ],
      }}
    >
      <Button icon={<MoreOutlined />}>Columns</Button>
    </Dropdown>
  );

  return (
    <div className="bg-white dark:bg-gray-950 p-2 rounded-lg border border-gray-200 dark:border-gray-800 mb-4 flex flex-col sm:flex-row sm:items-center sm:justify-between gap-4">
      <div className="flex flex-row items-center gap-2 w-full sm:w-auto">
        {viewMode &&
          onViewModeChange &&
          typeof window !== 'undefined' &&
          window.innerWidth >= 640 && (
            <Segmented
              size="middle"
              value={viewMode}
              onChange={(val) => onViewModeChange(val as 'table' | 'grid')}
              options={[
                { value: 'table', icon: <TableOutlined /> },
                { value: 'grid', icon: <AppstoreOutlined /> },
              ]}
              className="dark:!bg-gray-950"
            />
          )}

        <Search
          placeholder={searchPlaceholder}
          allowClear
          onChange={(e) => handleSearch(e.target.value)}
          value={searchTerm}
          className="w-full sm:w-[320px]"
          style={{ width: '100%' }}
        />
      </div>

      <Row gutter={8} align="middle" wrap={false}>
        {primaryAction && (
          <Col>
            {secondaryActions.length === 0 ? (
              <Button
                type="primary"
                data-cy={`control-action-${primaryAction.key}`}
                onClick={() =>
                  primaryAction.handler({
                    selected: selectedRowKeys,
                    refresh: () => {},
                  })
                }
              >
                {primaryAction.icon} {primaryAction.label}
              </Button>
            ) : (
              <Space.Compact>
                <Button
                  type="primary"
                  data-cy={`control-action-${primaryAction.key}`}
                  onClick={() =>
                    primaryAction.handler({
                      selected: selectedRowKeys,
                      refresh: () => {},
                    })
                  }
                >
                  {primaryAction.icon} {primaryAction.label}
                </Button>

                <Dropdown
                  data-cy="control-action-dropdown"
                  menu={{
                    items: secondaryActions.map((a) => ({
                      key: a.key,
                      label: <span data-cy={`control-action-${a.key}`}>{a.label}</span>,
                      icon: a.icon,
                      onClick: a.confirm
                        ? undefined
                        : () =>
                            a.handler({
                              selected: selectedRowKeys,
                              refresh: () => {},
                            }),
                    })),
                  }}
                  placement="bottomRight"
                >
                  <Button type="primary" icon={<MoreOutlined />} />
                </Dropdown>
              </Space.Compact>
            )}
          </Col>
        )}

        {hasBulk && resolvedPrimaryBulk && (
          <Col>
            {secondaryBulkActions.length === 0 ? (
              <Button
                data-cy={`bulk-action-${resolvedPrimaryBulk.key}`}
                onClick={() =>
                  resolvedPrimaryBulk.handler({
                    selected: selectedRowKeys,
                    refresh: () => {},
                  })
                }
              >
                {resolvedPrimaryBulk.icon} {resolvedPrimaryBulk.label}
              </Button>
            ) : (
              <Space.Compact>
                <Button
                  data-cy={`bulk-action-${resolvedPrimaryBulk.key}`}
                  onClick={() =>
                    resolvedPrimaryBulk.handler({
                      selected: selectedRowKeys,
                      refresh: () => {},
                    })
                  }
                >
                  {resolvedPrimaryBulk.icon} {resolvedPrimaryBulk.label}
                </Button>
                <Dropdown
                  data-cy="bulk-action-dropdown"
                  menu={{
                    items: secondaryBulkActions.map((a) => ({
                      key: a.key,
                      label: <span data-cy={`bulk-action-${a.key}`}>{a.label}</span>,
                      icon: a.icon,
                      onClick: a.confirm
                        ? undefined
                        : () =>
                            a.handler({
                              selected: selectedRowKeys,
                              refresh: () => {},
                            }),
                    })),
                  }}
                  placement="bottomRight"
                >
                  <Button icon={<MoreOutlined />} />
                </Dropdown>
              </Space.Compact>
            )}
          </Col>
        )}

        {hasActiveFilters && clearMenuItems.length > 0 && (
          <Col>
            {clearMenuItems.length === 1 ? (
              <Button
                icon={<ReloadOutlined />}
                onClick={() => clearMenuItems[0].onClick?.({ key: clearMenuItems[0].key } as any)}
                data-cy={clearMenuItems[0].key}
              >
                {clearMenuItems[0].label}
              </Button>
            ) : (
              <Dropdown menu={{ items: clearMenuItems }}>
                <Button icon={<ReloadOutlined />}>Clear</Button>
              </Dropdown>
            )}
          </Col>
        )}

        {columnToggleEnabled && columns?.length && <Col>{columnToggleMenu}</Col>}
      </Row>

      <Modal
        open={sortModalOpen}
        title="Sort Options"
        onCancel={() => setSortModalOpen(false)}
        footer={null}
        centered
      >
        <div className="space-y-4">
          {sortOptions.map((opt) => {
            const field = opt.field;
            const active = currentSort?.find((s) => s.startsWith(`${field}.`));
            const currentOrder = active?.split('.')[1] || 'none';

            return (
              <div key={field} className="grid grid-cols-1 sm:grid-cols-2 gap-2 items-center">
                <span className="font-medium truncate">{opt.label}</span>
                <Select
                  value={currentOrder}
                  className="w-full"
                  onChange={(order) => {
                    const updated = currentSort?.filter((s) => !s.startsWith(`${field}.`)) || [];
                    if (order !== 'none') updated.push(`${field}.${order}`);
                    onSortChange?.(updated);
                  }}
                  options={[
                    { label: 'None', value: 'none' },
                    { label: 'Ascending', value: 'ascend' },
                    { label: 'Descending', value: 'descend' },
                  ]}
                />
              </div>
            );
          })}
        </div>
      </Modal>

      <Modal
        open={filterModalOpen}
        title="Filters"
        onCancel={() => setFilterModalOpen(false)}
        footer={null}
        centered
      >
        <div className="space-y-4">
          {filterGroups.map((group) => {
            const value =
              activeFilters.find((f) => f.startsWith(`${group.key}:`))?.split(':')[1] || '';
            const values = activeFilters
              .filter((f) => f.startsWith(`${group.key}:`))
              .map((f) => f.split(':')[1]);

            return (
              <div key={group.key} className="space-y-1">
                <label className="block font-medium">{group.label}</label>

                {group.type === 'select' && group.options && (
                  <Select
                    className="w-full"
                    placeholder={`Select ${group.label}`}
                    value={value || undefined}
                    onChange={(v) => {
                      const updated = activeFilters.filter((f) => !f.startsWith(`${group.key}:`));
                      if (v) updated.push(`${group.key}:${v}`);
                      onFilterChange?.(updated);
                    }}
                    options={group.options}
                    allowClear
                  />
                )}

                {group.type === 'multi-select' && group.options && (
                  <Select
                    mode="multiple"
                    className="w-full"
                    placeholder={`Select ${group.label}`}
                    value={values}
                    onChange={(vals) => {
                      const updated = activeFilters.filter((f) => !f.startsWith(`${group.key}:`));
                      vals.forEach((v) => updated.push(`${group.key}:${v}`));
                      onFilterChange?.(updated);
                    }}
                    options={group.options}
                  />
                )}

                {group.type === 'text' && (
                  <Input
                    placeholder={`Enter ${group.label}`}
                    value={value}
                    onChange={(e) => {
                      const updated = activeFilters.filter((f) => !f.startsWith(`${group.key}:`));
                      const val = e.target.value;
                      if (val) updated.push(`${group.key}:${val}`);
                      onFilterChange?.(updated);
                    }}
                  />
                )}

                {group.type === 'number' && (
                  <Input
                    type="number"
                    placeholder={`Enter ${group.label}`}
                    value={value}
                    onChange={(e) => {
                      const updated = activeFilters.filter((f) => !f.startsWith(`${group.key}:`));
                      const val = e.target.value;
                      if (val) updated.push(`${group.key}:${val}`);
                      onFilterChange?.(updated);
                    }}
                  />
                )}
              </div>
            );
          })}
        </div>
      </Modal>
    </div>
  );
};

export default ControlBar;
