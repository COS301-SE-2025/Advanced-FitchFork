import React, { useState } from 'react';
import { Input, Button, Dropdown, Segmented, Col, Space, Checkbox, Popconfirm } from 'antd';
import {
  ReloadOutlined,
  TableOutlined,
  AppstoreOutlined,
  MoreOutlined,
  FilterOutlined,
  SortAscendingOutlined,
} from '@ant-design/icons';
import type { MenuItemType } from 'antd/es/menu/interface';
import type { EntityAction } from './EntityList';
import FilterModal from './common/FilterModal';
import SortModal from './common/SortModal';

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

  listMode?: boolean;
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
  listMode = false,
}: Props<T>) => {
  const [sortModalOpen, setSortModalOpen] = useState(false);
  const [filterModalOpen, setFilterModalOpen] = useState(false);

  const hasBulk = selectedRowKeys.length > 0 && bulkActions.length > 0;

  const hasSearch = !!searchTerm.trim();
  const hasSort = (currentSort?.length ?? 0) > 0;
  const hasFilters = (activeFilters?.length ?? 0) > 0;

  // show Clear control on mobile if any state is active
  const showClearInlineMobile = hasSearch || hasSort || hasFilters;

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
                    if (hiddenColumns?.has(col.key)) onToggleColumn?.(col.key);
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
                    if (!hiddenColumns?.has(col.key)) onToggleColumn?.(col.key);
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
                    if (shouldBeHidden !== currentlyHidden) onToggleColumn?.(col.key);
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

  // Reusable pieces
  const FiltersButton = (
    <Dropdown
      menu={{
        items: [
          ...(filterGroups.length > 0
            ? [
                {
                  key: 'open-filters',
                  icon: <FilterOutlined />,
                  label: <span data-cy="open-filter-modal">Filters</span>,
                  onClick: () => setFilterModalOpen(true),
                },
              ]
            : []),
          ...(sortOptions.length > 0
            ? [
                {
                  key: 'open-sort',
                  icon: <SortAscendingOutlined />,
                  label: <span data-cy="open-sort-modal">Sort</span>,
                  onClick: () => setSortModalOpen(true),
                },
              ]
            : []),
        ],
      }}
      trigger={['click']}
    >
      <Button data-cy="filters-dropdown" className="whitespace-nowrap">
        Filters
      </Button>
    </Dropdown>
  );

  const ClearControl = (
    <>
      {clearMenuItems.length === 1 ? (
        <Button
          icon={<ReloadOutlined />}
          onClick={() => clearMenuItems[0].onClick?.({ key: clearMenuItems[0].key } as any)}
          data-cy={clearMenuItems[0].key}
          className="w-full sm:w-auto"
        >
          {clearMenuItems[0].label}
        </Button>
      ) : (
        <Dropdown menu={{ items: clearMenuItems }}>
          <Button icon={<ReloadOutlined />} className="w-full sm:w-auto">
            Clear
          </Button>
        </Dropdown>
      )}
    </>
  );

  const ActionsGroup = (
    <div className="w-full sm:w-auto !m-0 flex flex-wrap items-center gap-2">
      {primaryAction && (
        <div className="w-full sm:w-auto">
          <Space.Compact className="w-full sm:w-auto">
            {primaryAction.confirm ? (
              <Popconfirm
                title={`Are you sure you want to ${primaryAction.label.toLowerCase()}?`}
                okText="Yes"
                cancelText="Cancel"
                placement="topRight"
                onConfirm={() =>
                  primaryAction.handler({
                    selected: selectedRowKeys,
                    refresh: () => {},
                  })
                }
              >
                <Button
                  type="primary"
                  data-cy={`control-action-${primaryAction.key}`}
                  className="w-full sm:w-auto"
                >
                  {primaryAction.icon} {primaryAction.label}
                </Button>
              </Popconfirm>
            ) : (
              <Button
                type="primary"
                data-cy={`control-action-${primaryAction.key}`}
                onClick={() =>
                  primaryAction.handler({
                    selected: selectedRowKeys,
                    refresh: () => {},
                  })
                }
                className="w-full sm:w-auto"
              >
                {primaryAction.icon} {primaryAction.label}
              </Button>
            )}

            {secondaryActions.length > 0 && (
              <Dropdown
                data-cy="control-action-dropdown"
                menu={{
                  items: secondaryActions.map((a) => ({
                    key: a.key,
                    icon: a.icon,
                    label: a.confirm ? (
                      <Popconfirm
                        title={`Are you sure you want to ${a.label.toLowerCase()}?`}
                        okText="Yes"
                        cancelText="Cancel"
                        placement="topRight"
                        onConfirm={() =>
                          a.handler({
                            selected: selectedRowKeys,
                            refresh: () => {},
                          })
                        }
                      >
                        <span data-cy={`control-action-${a.key}`}>{a.label}</span>
                      </Popconfirm>
                    ) : (
                      <span
                        data-cy={`control-action-${a.key}`}
                        onClick={() =>
                          a.handler({
                            selected: selectedRowKeys,
                            refresh: () => {},
                          })
                        }
                      >
                        {a.label}
                      </span>
                    ),
                  })),
                }}
                placement="bottomRight"
              >
                <Button type="primary" icon={<MoreOutlined />} />
              </Dropdown>
            )}
          </Space.Compact>
        </div>
      )}

      {hasBulk && resolvedPrimaryBulk && (
        <div className="w-full sm:w-auto">
          {secondaryBulkActions.length === 0 ? (
            <Button
              data-cy={`bulk-action-${resolvedPrimaryBulk.key}`}
              onClick={() =>
                resolvedPrimaryBulk.handler({
                  selected: selectedRowKeys,
                  refresh: () => {},
                })
              }
              className="w-full sm:w-auto"
            >
              {resolvedPrimaryBulk.icon} {resolvedPrimaryBulk.label}
            </Button>
          ) : (
            <Space.Compact className="w-full sm:w-auto">
              <Button
                data-cy={`bulk-action-${resolvedPrimaryBulk.key}`}
                onClick={() =>
                  resolvedPrimaryBulk.handler({
                    selected: selectedRowKeys,
                    refresh: () => {},
                  })
                }
                className="w-full sm:w-auto"
              >
                {resolvedPrimaryBulk.icon} {resolvedPrimaryBulk.label}
              </Button>
              <Dropdown
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
                <Button icon={<MoreOutlined />} data-cy="bulk-action-dropdown" />
              </Dropdown>
            </Space.Compact>
          )}
        </div>
      )}

      {columnToggleEnabled && columns?.length && !listMode && <div>{columnToggleMenu}</div>}
    </div>
  );

  return (
    <div className="bg-white dark:bg-gray-900 p-2 rounded-lg border border-gray-200 dark:border-gray-800 mb-4 flex flex-col sm:flex-row sm:items-center sm:justify-between gap-4">
      {/* LEFT: view toggle (desktop only), search, filters */}
      <div className="flex items-center gap-2 w-full sm:w-auto">
        {/* View toggle: absolutely hidden on mobile via wrapper */}
        {viewMode && onViewModeChange && (
          <div className="hidden sm:flex">
            <Segmented
              size="middle"
              value={viewMode}
              onChange={(val) => onViewModeChange(val as 'table' | 'grid')}
              options={[
                {
                  value: 'table',
                  label: (
                    <span data-cy="view-toggle-table">
                      <TableOutlined />
                    </span>
                  ),
                },
                {
                  value: 'grid',
                  label: (
                    <span data-cy="view-toggle-grid">
                      <AppstoreOutlined />
                    </span>
                  ),
                },
              ]}
              className="dark:!bg-gray-950"
            />
          </div>
        )}

        {/* Search grows to fill on mobile */}
        <Search
          placeholder={searchPlaceholder}
          allowClear
          onChange={(e) => handleSearch(e.target.value)}
          value={searchTerm}
          className="flex-1 sm:w-[320px]"
          data-cy="entity-search"
        />

        {/* Filters button:
            - Mobile: always show if sorts/filters exist
            - Desktop: follow original condition (grid or listMode) */}
        <div className="sm:hidden">
          {(filterGroups.length > 0 || sortOptions.length > 0) && FiltersButton}
        </div>

        <div className="hidden sm:block">
          {(filterGroups.length > 0 || sortOptions.length > 0) &&
            (viewMode === 'grid' || listMode) &&
            FiltersButton}
        </div>

        {/* Desktop Clear (left area); mobile version is below next to actions */}
        <div className="hidden sm:block">
          {clearMenuItems.length > 0 && <Col>{ClearControl}</Col>}
        </div>
      </div>

      {/* RIGHT: Actions (desktop) */}
      <div className="hidden sm:block">{ActionsGroup}</div>

      {/* MOBILE BELOW:
         - If Clear exists -> Actions + Clear in a 2-col grid (50/50)
         - If no Clear -> render Actions alone (no grid, no extra gap) */}
      {(primaryAction ||
        hasBulk ||
        (columnToggleEnabled && columns?.length) ||
        showClearInlineMobile) &&
        (showClearInlineMobile ? (
          <div className="sm:hidden grid grid-cols-2 gap-2 w-full">
            <div className="col-span-1">{ActionsGroup}</div>
            <div className="col-span-1 flex justify-end items-center">{ClearControl}</div>
          </div>
        ) : (
          <div className="sm:hidden w-full">{ActionsGroup}</div>
        ))}

      {/* Sort Modal (extracted) */}
      <SortModal
        open={sortModalOpen}
        onClose={() => setSortModalOpen(false)}
        sortOptions={sortOptions}
        currentSort={currentSort}
        onChange={(val) => onSortChange?.(val)}
      />

      {/* Filter Modal (extracted) */}
      <FilterModal
        open={filterModalOpen}
        onClose={() => setFilterModalOpen(false)}
        filterGroups={filterGroups}
        activeFilters={activeFilters}
        onChange={(vals) => onFilterChange?.(vals)}
      />
    </div>
  );
};

export default ControlBar;
