import { Table, Empty, Button, Dropdown, Popconfirm, Space, Tooltip } from 'antd';
import { ReloadOutlined, MoreOutlined } from '@ant-design/icons';
import { forwardRef, useEffect, useImperativeHandle, useState } from 'react';
import type { ColumnsType } from 'antd/es/table';
import type { SortOption } from '@/types/common';

import ControlBar from '@/components/ControlBar';
import TagSummary from '@/components/TagSummary';
import { useNotifier } from '@/components/Notifier';
import { useEntityViewState } from '@/hooks/useEntityViewState';
import type { ModuleRole } from '@/types/modules';

export type EntityAction<T> = {
  key: string;
  label: string;
  icon?: React.ReactNode;
  isPrimary?: boolean;
  confirm?: boolean;
  handler: (context: { entity?: T; refresh: () => void; selected?: React.Key[] }) => void;
};

export type EntityColumnType<T> = ColumnsType<T>[number] & {
  defaultHidden?: boolean;
  hiddenFor?: ModuleRole[];
};

export type EntityListProps<T> = {
  name: string;
  viewModeKey?: string;
  defaultViewMode?: 'table' | 'grid';
  fetchItems: (params: {
    page: number;
    per_page: number;
    query: string;
    sort: SortOption[];
    filters: Record<string, string[]>;
  }) => Promise<{ items: T[]; total: number }>;
  columns: EntityColumnType<T>[];
  getRowKey: (item: T) => string | number;
  onRowClick?: (item: T) => void;
  renderGridItem?: (item: T, actions: React.ReactNode[]) => React.ReactNode;
  actions?: {
    entity?: (entity: T) => EntityAction<T>[];
    control?: EntityAction<T>[];
    bulk?: EntityAction<T>[];
  };
  columnToggleEnabled?: boolean;
};

export type EntityListHandle = {
  refresh: () => void;
  clearSelection: () => void;
  getSelectedRowKeys: () => React.Key[];
};

const EntityList = forwardRef(function <T>(
  props: EntityListProps<T>,
  ref: React.Ref<EntityListHandle>,
) {
  const {
    name,
    defaultViewMode = 'table',
    fetchItems,
    viewModeKey = `${name.toLowerCase().replace(/\s+/g, '_')}_view_mode`,
    columns,
    getRowKey,
    onRowClick,
    renderGridItem,
    actions,
    columnToggleEnabled = false,
  } = props;

  const {
    viewMode,
    setViewMode,
    selectedRowKeys,
    setSelectedRowKeys,
    searchTerm,
    setSearchTerm,
    sorterState,
    setSorterState,
    filterState,
    setFilterState,
    pagination,
    setPagination,
    clearSearch,
  } = useEntityViewState<T>({
    viewModeKey,
    defaultViewMode,
    getInitialNewItem: () => ({}) as Partial<T>,
  });

  const { notifyError } = useNotifier();
  const [loading, setLoading] = useState(false);
  const [items, setItems] = useState<T[]>([]);
  const [hiddenColumns, setHiddenColumns] = useState<Set<string>>(
    new Set(columns.filter((col) => col.defaultHidden).map((col) => col.key as string)),
  );

  const toggleColumn = (key: string) => {
    setHiddenColumns((prev) => {
      const next = new Set(prev);
      if (next.has(key)) next.delete(key);
      else next.add(key);
      return next;
    });
  };

  const fetchData = async () => {
    setLoading(true);
    const res = await fetchItems({
      page: pagination.current,
      per_page: pagination.pageSize,
      query: searchTerm,
      sort: sorterState,
      filters: filterState,
    });

    if (res) {
      setItems(res.items);
      setPagination({ total: res.total });
    } else {
      notifyError('Fetch Failed', 'Could not load data');
    }

    setLoading(false);
  };

  useImperativeHandle(ref, () => ({
    refresh: fetchData,
    clearSelection: () => setSelectedRowKeys([]),
    getSelectedRowKeys: () => selectedRowKeys,
  }));

  useEffect(() => {
    fetchData();
  }, [searchTerm, filterState, sorterState, pagination.current, pagination.pageSize]);

  const hasSearch = !!searchTerm.trim();
  const hasSort = sorterState.length > 0;
  const hasFilters = Object.keys(filterState).length > 0;

  const clearMenuItems = [
    hasSearch && {
      key: 'clear-search',
      label: 'Clear Search',
      onClick: () => setSearchTerm(''),
    },
    hasSort && {
      key: 'clear-sort',
      label: 'Clear Sort',
      onClick: () => setSorterState([]),
    },
    hasFilters && {
      key: 'clear-filters',
      label: 'Clear Filters',
      onClick: () => setFilterState({}),
    },
  ].filter(Boolean) as {
    key: string;
    label: string;
    onClick: () => void;
  }[];

  if (clearMenuItems.length > 1) {
    clearMenuItems.push({
      key: 'clear-all',
      label: 'Clear All',
      onClick: () => {
        setSearchTerm('');
        setSorterState([]);
        setFilterState({});
      },
    });
  }

  const controlActions = actions?.control ?? [];
  const bulkActions = actions?.bulk ?? [];

  const controlledColumns: EntityColumnType<T>[] = columns.map((col) => {
    const sortState = sorterState.find((s) => s.field === col.key);
    const filterStateForCol = filterState[col.key as string];
    return {
      ...col,
      sortOrder: sortState?.order,
      filteredValue: filterStateForCol ?? null,
    };
  });

  const extendedColumns: EntityColumnType<T>[] = controlledColumns.filter(
    (col) => !hiddenColumns.has(col.key as string),
  );

  if (actions?.entity) {
    extendedColumns.push({
      title: 'Actions',
      key: 'actions',
      align: 'right',
      width: 140,
      render: (_, record) => {
        const entityActions = actions.entity!(record);
        if (!entityActions.length) return null;

        const resolvedPrimary = entityActions.find((a) => a.isPrimary) ?? entityActions[0];
        const secondaryActions = entityActions.filter((a) => a.key !== resolvedPrimary.key);

        return (
          <div onClick={(e) => e.stopPropagation()}>
            {secondaryActions.length === 0 ? (
              <Button
                size="small"
                icon={resolvedPrimary.icon}
                onClick={() => resolvedPrimary.handler({ entity: record, refresh: fetchData })}
              >
                {resolvedPrimary.label}
              </Button>
            ) : (
              <Space.Compact>
                <Button
                  size="small"
                  icon={resolvedPrimary.icon}
                  onClick={() => resolvedPrimary.handler({ entity: record, refresh: fetchData })}
                >
                  {resolvedPrimary.label}
                </Button>
                <Dropdown
                  menu={{
                    items: secondaryActions.map((a) => ({
                      key: a.key,
                      label: a.confirm ? (
                        <Popconfirm
                          title={`Are you sure you want to ${a.label.toLowerCase()}?`}
                          okText="Yes"
                          cancelText="No"
                          onConfirm={() => a.handler({ entity: record, refresh: fetchData })}
                        >
                          <span>{a.label}</span>
                        </Popconfirm>
                      ) : (
                        a.label
                      ),
                      icon: a.icon,
                      onClick: a.confirm
                        ? undefined
                        : () => a.handler({ entity: record, refresh: fetchData }),
                    })),
                  }}
                  placement="bottomRight"
                >
                  <Button size="small" icon={<MoreOutlined />} />
                </Dropdown>
              </Space.Compact>
            )}
          </div>
        );
      },
    });
  }

  return (
    <div>
      <ControlBar
        handleSearch={setSearchTerm}
        searchTerm={searchTerm}
        viewMode={renderGridItem ? viewMode : undefined}
        onViewModeChange={renderGridItem ? setViewMode : undefined}
        selectedRowKeys={selectedRowKeys}
        searchPlaceholder={`Search ${name.toLowerCase()}`}
        sortOptions={columns
          .filter((c) => c.sorter)
          .map((c) => ({ label: c.title as string, field: c.key as string }))}
        currentSort={sorterState.map((s) => `${s.field}.${s.order}`)}
        onSortChange={(vals) =>
          setSorterState(
            vals.map((v) => {
              const [field, order] = v.split('.');
              return { field, order: order as 'ascend' | 'descend' };
            }),
          )
        }
        filterGroups={columns
          .filter((c) => c.filters)
          .map((c) => ({
            key: c.key as string,
            label: c.title as string,
            type: 'select',
            options: (c.filters ?? []).map((f) => ({
              label: f.text as string,
              value: f.value as string,
            })),
          }))}
        activeFilters={Object.entries(filterState ?? {}).flatMap(([k, vals]) =>
          Array.isArray(vals) ? vals.map((v) => `${k}:${v}`) : [],
        )}
        onFilterChange={(vals) => {
          const grouped: Record<string, string[]> = {};
          vals.forEach((v) => {
            const [key, val] = v.split(':');
            if (!grouped[key]) grouped[key] = [];
            grouped[key].push(val);
          });
          setFilterState(grouped);
        }}
        actions={controlActions}
        bulkActions={bulkActions}
        columnToggleEnabled={columnToggleEnabled && viewMode !== 'grid'}
        columns={columns.map((col) => ({
          key: col.key as string,
          label:
            typeof col.title === 'function'
              ? String(col.key)
              : (col.title as string) || String(col.key),
          defaultHidden: !!col.defaultHidden,
        }))}
        hiddenColumns={hiddenColumns}
        onToggleColumn={toggleColumn}
      />

      <TagSummary
        searchTerm={searchTerm}
        onClearSearch={clearSearch}
        filters={filterState}
        onClearFilter={(key) => {
          const updated = { ...filterState };
          delete updated[key];
          setFilterState(updated);
        }}
        sorters={sorterState.map((s) => ({ columnKey: s.field, order: s.order }))}
        onClearSorter={(key) => setSorterState(sorterState.filter((s) => s.field !== key))}
      />

      {viewMode === 'grid' && renderGridItem ? (
        <div className="grid gap-4 grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4 mt-4">
          {items.map((item) => {
            const allActions = actions?.entity?.(item) ?? [];
            const inlineLimit = allActions.length >= 4 ? 2 : 3;
            const inlineActions = allActions.slice(0, inlineLimit);
            const dropdownActions = allActions.slice(inlineLimit);

            const actionButtons = [
              ...inlineActions.map((a) => (
                <Tooltip title={a.label} key={a.key}>
                  {a.confirm ? (
                    <Popconfirm
                      title={`Are you sure you want to ${a.label.toLowerCase()}?`}
                      okText="Yes"
                      cancelText="No"
                      onConfirm={(e) => {
                        e?.stopPropagation?.();
                        a.handler({ entity: item, refresh: fetchData, selected: selectedRowKeys });
                      }}
                      onCancel={(e) => {
                        e?.stopPropagation?.();
                      }}
                    >
                      <Button icon={a.icon} type="text" onClick={(e) => e.stopPropagation()} />
                    </Popconfirm>
                  ) : (
                    <Button
                      icon={a.icon}
                      type="text"
                      onClick={(e) => {
                        e.stopPropagation();
                        a.handler({ entity: item, refresh: fetchData, selected: selectedRowKeys });
                      }}
                    />
                  )}
                </Tooltip>
              )),
            ];

            if (dropdownActions.length > 0) {
              actionButtons.push(
                <Dropdown
                  key="more"
                  menu={{
                    items: dropdownActions.map((a) => ({
                      key: a.key,
                      label: a.confirm ? (
                        <Popconfirm
                          title={`Are you sure you want to ${a.label.toLowerCase()}?`}
                          okText="Yes"
                          cancelText="No"
                          onConfirm={() =>
                            a.handler({
                              entity: item,
                              refresh: fetchData,
                              selected: selectedRowKeys,
                            })
                          }
                          onCancel={(e) => {
                            e?.stopPropagation?.();
                          }}
                        >
                          <span>{a.label}</span>
                        </Popconfirm>
                      ) : (
                        <span
                          onClick={() =>
                            a.handler({
                              entity: item,
                              refresh: fetchData,
                              selected: selectedRowKeys,
                            })
                          }
                        >
                          {a.label}
                        </span>
                      ),
                      icon: a.icon,
                    })),
                  }}
                  placement="bottomRight"
                >
                  <Button type="text" icon={<MoreOutlined />} />
                </Dropdown>,
              );
            }

            return <div key={getRowKey(item)}>{renderGridItem(item, actionButtons)}</div>;
          })}
        </div>
      ) : (
        <Table<T>
          columns={extendedColumns}
          dataSource={items}
          rowKey={getRowKey}
          loading={loading}
          pagination={{
            ...pagination,
            showSizeChanger: true,
            showQuickJumper: true,
            onChange: (page, pageSize) => setPagination({ current: page, pageSize }),
          }}
          rowSelection={
            bulkActions.length > 0
              ? {
                  selectedRowKeys,
                  onChange: setSelectedRowKeys,
                }
              : undefined
          }
          onChange={(pagination, filters, sorter) => {
            const sorterArray = (Array.isArray(sorter) ? sorter : [sorter])
              .filter(
                (s): s is { columnKey: string; order: 'ascend' | 'descend' } =>
                  !!s.columnKey && !!s.order,
              )
              .map((s) => ({ field: String(s.columnKey), order: s.order }));
            setSorterState(sorterArray);
            setFilterState(filters as Record<string, string[]>);
            setPagination({
              current: pagination.current || 1,
              pageSize: pagination.pageSize || 10,
            });
          }}
          onRow={onRowClick ? (record) => ({ onClick: () => onRowClick(record) }) : undefined}
          locale={{
            emptyText: (
              <Empty image={Empty.PRESENTED_IMAGE_SIMPLE} description="No data found.">
                {clearMenuItems.length === 1 ? (
                  <Button icon={<ReloadOutlined />} onClick={clearMenuItems[0].onClick}>
                    {clearMenuItems[0].label}
                  </Button>
                ) : (
                  <Dropdown
                    menu={{
                      items: clearMenuItems.map((item) => ({
                        key: item.key,
                        label: item.label,
                        onClick: item.onClick,
                      })),
                    }}
                  >
                    <Button icon={<ReloadOutlined />}>Clear</Button>
                  </Dropdown>
                )}
              </Empty>
            ),
          }}
          className="bg-white dark:bg-gray-950 border-1 border-gray-200 dark:border-gray-800 rounded-lg overflow-hidden"
        />
      )}
    </div>
  );
}) as <T>(props: EntityListProps<T> & { ref?: React.Ref<EntityListHandle> }) => JSX.Element;

export { EntityList };
