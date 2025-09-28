import React, { useState } from 'react';
import {
  Input,
  Button,
  Dropdown,
  Segmented,
  Col,
  Space,
  Checkbox,
  Popconfirm,
  Badge,
  Tooltip,
} from 'antd';
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
import { useUI } from '@/context/UIContext';

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
  onRefreshClick?: () => void;
  refreshBadgeCount?: number;
  refreshBadgeTooltip?: string;
}

const ControlBar = <T,>(props: Props<T>) => {
  const {
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
    onRefreshClick,
    refreshBadgeCount,
    refreshBadgeTooltip,
  } = props;

  const { isSm } = useUI();
  const isDesktop = isSm;

  const [sortModalOpen, setSortModalOpen] = useState(false);
  const [filterModalOpen, setFilterModalOpen] = useState(false);

  const hasBulk = (selectedRowKeys?.length ?? 0) > 0 && (bulkActions?.length ?? 0) > 0;
  const hasSearch = !!searchTerm.trim();
  const hasSort = (currentSort?.length ?? 0) > 0;
  const hasFilters = (activeFilters?.length ?? 0) > 0;

  const clearMenuItems: MenuItemType[] = [
    hasSearch && {
      key: 'clear-search',
      label: <span data-testid="clear-search">Clear Search</span>,
      onClick: () => handleSearch(''),
    },
    hasSort && {
      key: 'clear-sort',
      label: <span data-testid="clear-sort">Clear Sort</span>,
      onClick: () => onSortChange?.([]),
    },
    hasFilters && {
      key: 'clear-filters',
      label: <span data-testid="clear-filters">Clear Filters</span>,
      onClick: () => onFilterChange?.([]),
    },
  ].filter(Boolean) as MenuItemType[];

  if (clearMenuItems.length > 1) {
    clearMenuItems.push({
      key: 'clear-all',
      label: <span data-testid="clear-all">Clear All</span>,
      onClick: () => {
        handleSearch('');
        onSortChange?.([]);
        onFilterChange?.([]);
      },
    });
  }

  const primaryAction = actions.find((a) => a.isPrimary) ?? actions[0] ?? null;
  const secondaryActions = primaryAction ? actions.filter((a) => a.key !== primaryAction.key) : [];

  const resolvedPrimaryBulk = hasBulk
    ? (bulkActions.find((a) => a.isPrimary) ?? bulkActions[0] ?? null)
    : null;
  const secondaryBulkActions = resolvedPrimaryBulk
    ? bulkActions.filter((a) => a.key !== resolvedPrimaryBulk.key)
    : [];

  const columnsMenuItems: MenuItemType[] = [
    ...(columns?.map((col) => ({
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
    })) ?? []),
    { type: 'divider' } as any,
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
  ];

  const FiltersButton = ({ block = false }: { block?: boolean }) => (
    <Dropdown
      menu={{
        items: [
          ...(filterGroups.length > 0
            ? [
                {
                  key: 'open-filters',
                  icon: <FilterOutlined />,
                  label: <span data-testid="open-filter-modal">Filters</span>,
                  onClick: () => setFilterModalOpen(true),
                },
              ]
            : []),
          ...(sortOptions.length > 0
            ? [
                {
                  key: 'open-sort',
                  icon: <SortAscendingOutlined />,
                  label: <span data-testid="open-sort-modal">Sort</span>,
                  onClick: () => setSortModalOpen(true),
                },
              ]
            : []),
        ],
      }}
      trigger={['click']}
    >
      <Button
        data-testid="filters-dropdown"
        className={`whitespace-nowrap flex items-center gap-1 ${block ? 'w-full justify-center' : ''}`}
      >
        <FilterOutlined />
        {isDesktop ? <span>Filters</span> : <span>Filters & Sort</span>}
      </Button>
    </Dropdown>
  );

  const RefreshBtn = ({ block = false }: { block?: boolean }) => (
    <Tooltip
      title={refreshBadgeCount ? (refreshBadgeTooltip ?? `${refreshBadgeCount} new`) : undefined}
    >
      <Button
        icon={<ReloadOutlined />}
        onClick={onRefreshClick}
        data-testid="refresh-button"
        className={block ? 'w-full justify-center' : undefined}
      >
        <span className="inline-flex items-center gap-2">
          <span>Refresh</span>
          {typeof refreshBadgeCount === 'number' && refreshBadgeCount > 0 ? (
            <Badge count={Math.min(refreshBadgeCount, 99)} overflowCount={99} />
          ) : null}
        </span>
      </Button>
    </Tooltip>
  );

  const ClearControl = (
    <>
      {clearMenuItems.length === 1 ? (
        <Button
          icon={<ReloadOutlined />}
          onClick={() => clearMenuItems[0].onClick?.({ key: clearMenuItems[0].key } as any)}
          data-testid={clearMenuItems[0].key}
          className={isDesktop ? undefined : 'w-full justify-center'}
        >
          {clearMenuItems[0].label}
        </Button>
      ) : (
        <Dropdown menu={{ items: clearMenuItems }}>
          <Button
            className={isDesktop ? undefined : 'w-full justify-center'}
            icon={<ReloadOutlined />}
          >
            Clear
          </Button>
        </Dropdown>
      )}
    </>
  );

  const ActionsGroup = (
    <div className={isDesktop ? 'flex items-center gap-2' : 'w-full flex flex-col gap-2'}>
      {primaryAction && (
        <div className={isDesktop ? undefined : 'w-full'}>
          <Space.Compact className={isDesktop ? undefined : 'w-full'}>
            {primaryAction.confirm ? (
              <Popconfirm
                title={`Are you sure you want to ${primaryAction.label.toLowerCase()}?`}
                okText="Yes"
                cancelText="No"
                okButtonProps={{ 'data-testid': 'confirm-yes' }}
                cancelButtonProps={{ 'data-testid': 'confirm-no' }}
                placement="topRight"
                onConfirm={() =>
                  primaryAction.handler({ selected: selectedRowKeys, refresh: () => {} })
                }
              >
                <Button
                  type="primary"
                  data-testid={`control-action-${primaryAction.key}`}
                  className={isDesktop ? undefined : 'w-full'}
                >
                  {primaryAction.icon} {primaryAction.label}
                </Button>
              </Popconfirm>
            ) : (
              <Button
                type="primary"
                data-testid={`control-action-${primaryAction.key}`}
                onClick={() =>
                  primaryAction.handler({ selected: selectedRowKeys, refresh: () => {} })
                }
                className={isDesktop ? undefined : 'w-full'}
              >
                {primaryAction.icon} {primaryAction.label}
              </Button>
            )}

            {secondaryActions.length > 0 && (
              <Dropdown
                data-testid="control-action-dropdown"
                menu={{
                  items: secondaryActions.map((a) => ({
                    key: a.key,
                    icon: a.icon,
                    label: a.confirm ? (
                      <Popconfirm
                        title={`Are you sure you want to ${a.label.toLowerCase()}?`}
                        okText="Yes"
                        cancelText="No"
                        okButtonProps={{ 'data-testid': 'confirm-yes' }}
                        cancelButtonProps={{ 'data-testid': 'confirm-no' }}
                        placement="topRight"
                        onConfirm={() =>
                          a.handler({ selected: selectedRowKeys, refresh: () => {} })
                        }
                      >
                        <span data-testid={`control-action-${a.key}`}>{a.label}</span>
                      </Popconfirm>
                    ) : (
                      <span
                        data-testid={`control-action-${a.key}`}
                        onClick={() => a.handler({ selected: selectedRowKeys, refresh: () => {} })}
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
        <div className={isDesktop ? undefined : 'w-full'}>
          {secondaryBulkActions.length === 0 ? (
            <Button
              data-testid={`bulk-action-${resolvedPrimaryBulk.key}`}
              onClick={() =>
                resolvedPrimaryBulk.handler({ selected: selectedRowKeys, refresh: () => {} })
              }
              className={isDesktop ? undefined : 'w-full'}
            >
              {resolvedPrimaryBulk.icon} {resolvedPrimaryBulk.label}
            </Button>
          ) : (
            <Space.Compact className={isDesktop ? undefined : 'w-full'}>
              <Button
                data-testid={`bulk-action-${resolvedPrimaryBulk.key}`}
                onClick={() =>
                  resolvedPrimaryBulk.handler({ selected: selectedRowKeys, refresh: () => {} })
                }
                className={isDesktop ? undefined : 'w-full'}
              >
                {resolvedPrimaryBulk.icon} {resolvedPrimaryBulk.label}
              </Button>
              <Dropdown
                menu={{
                  items: secondaryBulkActions.map((a) => ({
                    key: a.key,
                    label: <span data-testid={`bulk-action-${a.key}`}>{a.label}</span>,
                    icon: a.icon,
                    onClick: a.confirm
                      ? undefined
                      : () => a.handler({ selected: selectedRowKeys, refresh: () => {} }),
                  })),
                }}
                placement="bottomRight"
              >
                <Button icon={<MoreOutlined />} data-testid="bulk-action-dropdown" />
              </Dropdown>
            </Space.Compact>
          )}
        </div>
      )}
    </div>
  );

  const rootClasses = isDesktop
    ? 'mb-4 flex flex-row items-center justify-between gap-4 bg-white dark:bg-gray-900 p-2 rounded-lg border border-gray-200 dark:border-gray-800'
    : 'mb-4 flex flex-col gap-3';

  const leftClasses = isDesktop ? 'flex items-center gap-2 flex-1' : 'w-full flex flex-col gap-2';

  const hasActionsMobile =
    !isDesktop &&
    (!!primaryAction || !!resolvedPrimaryBulk || (columnToggleEnabled && !!columns?.length));
  const showClearMobile =
    !isDesktop && (hasSearch || hasSort || hasFilters) && clearMenuItems.length > 0;

  const searchClasses = isDesktop ? 'w-full max-w-[360px]' : 'w-full';
  const compactClasses = isDesktop ? '' : 'w-full';

  const hasFilterOrSort = filterGroups.length > 0 || sortOptions.length > 0;

  return (
    <div className={rootClasses}>
      {/* LEFT: view toggle (desktop), search, filters, clear (desktop inline) */}
      <div className={leftClasses}>
        {viewMode && onViewModeChange && isDesktop && (
          <Segmented
            size="middle"
            value={viewMode}
            onChange={(val) => onViewModeChange(val as 'table' | 'grid')}
            options={[
              {
                value: 'table',
                label: (
                  <span data-testid="view-toggle-table">
                    <TableOutlined />
                  </span>
                ),
              },
              {
                value: 'grid',
                label: (
                  <span data-testid="view-toggle-grid">
                    <AppstoreOutlined />
                  </span>
                ),
              },
            ]}
            className="dark:!bg-gray-950"
          />
        )}

        {isDesktop ? (
          // Desktop: always show Filters button if filters/sort exist (table, grid, or list)
          <Space.Compact className={compactClasses}>
            <Search
              placeholder={searchPlaceholder}
              allowClear
              onChange={(e) => handleSearch(e.target.value)}
              value={searchTerm}
              className={searchClasses}
              data-testid="entity-search"
            />
            {hasFilterOrSort && <FiltersButton />}
          </Space.Compact>
        ) : (
          // Mobile: stack full-width
          <div className="w-full flex flex-col gap-2">
            <Search
              placeholder={searchPlaceholder}
              allowClear
              onChange={(e) => handleSearch(e.target.value)}
              value={searchTerm}
              className="w-full"
              data-testid="entity-search"
            />
            {hasFilterOrSort && <FiltersButton block />}
            <RefreshBtn block />
          </div>
        )}

        {isDesktop && clearMenuItems.length > 0 && <Col>{ClearControl}</Col>}
      </div>

      {/* RIGHT: desktop actions + column toggle */}
      {isDesktop && (
        <div className="flex items-center gap-2">
          {ActionsGroup}
          <Space.Compact>
            <RefreshBtn />
            {columnToggleEnabled && columns?.length && !listMode && (
              <Dropdown
                menu={{ items: columnsMenuItems }}
                placement="bottomRight"
                trigger={['click']}
              >
                <Button icon={<MoreOutlined />} />
              </Dropdown>
            )}
          </Space.Compact>
        </div>
      )}

      {/* MOBILE: actions/clear below */}
      {!isDesktop && (hasActionsMobile || showClearMobile) && (
        <div className="w-full">
          {hasActionsMobile && showClearMobile ? (
            <div className="grid grid-cols-2 gap-2">
              <div className="col-span-1">{ActionsGroup}</div>
              <div className="col-span-1">{ClearControl}</div>
            </div>
          ) : hasActionsMobile ? (
            <div className="w-full">{ActionsGroup}</div>
          ) : (
            <div className="w-full">{ClearControl}</div>
          )}
        </div>
      )}

      {/* Modals */}
      <SortModal
        open={sortModalOpen}
        onClose={() => setSortModalOpen(false)}
        sortOptions={sortOptions}
        currentSort={currentSort}
        onChange={(val) => onSortChange?.(val)}
      />
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
