import { Table, Empty, Button, Dropdown, Popconfirm, Space, Tooltip, List } from 'antd';
import { ReloadOutlined, MoreOutlined } from '@ant-design/icons';
import { forwardRef, useEffect, useImperativeHandle, useState, type JSX } from 'react';
import type { ColumnsType } from 'antd/es/table';
import type { SortOption } from '@/types/common';

import { useNotifier } from '@/components/common';
import { useEntityViewState } from '@/hooks/useEntityViewState';
import type { ModuleRole } from '@/types/modules';
import ControlBar from './ControlBar';

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
  renderListItem?: (item: T) => React.ReactNode;
  listMode?: boolean;
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
    listMode = false,
    renderListItem,
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
  const [scrollHeight, setScrollHeight] = useState<number | undefined>();

  useEffect(() => {
    const updateHeight = () => {
      const viewportHeight = window.innerHeight;
      const tableTop =
        document.getElementById('scrollable-entity-table')?.getBoundingClientRect().top ?? 0;

      const footerHeight = 120; // estimated AntD pagination
      const padding = 32; // safety margin

      setScrollHeight(viewportHeight - tableTop - footerHeight - padding);
    };

    updateHeight();
    window.addEventListener('resize', updateHeight);
    return () => window.removeEventListener('resize', updateHeight);
  }, []);

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

  const goToPage = async (page: number) => {
    setLoading(true);
    const res = await fetchItems({
      page,
      per_page: pagination.pageSize,
      query: searchTerm,
      sort: sorterState,
      filters: filterState,
    });

    if (res) {
      setItems(res.items);
      setPagination({ current: page, total: res.total });
    } else {
      notifyError('Fetch Failed', 'Could not fetch data');
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

  // Reusable empty-state (matches table's style)
  const renderEmptyState = () => (
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
  );

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
          <div onClick={(e) => e.stopPropagation()} data-cy="entity-actions">
            {secondaryActions.length === 0 ? (
              <Button
                size="small"
                icon={resolvedPrimary.icon}
                data-cy={`entity-action-${resolvedPrimary.key}`}
                onClick={() => resolvedPrimary.handler({ entity: record, refresh: fetchData })}
              >
                {resolvedPrimary.label}
              </Button>
            ) : (
              <Space.Compact>
                {resolvedPrimary.confirm ? (
                  <Popconfirm
                    title={`Are you sure you want to ${resolvedPrimary.label.toLowerCase()}?`}
                    okText="Yes"
                    cancelText="No"
                    onConfirm={() =>
                      resolvedPrimary.handler({ entity: record, refresh: fetchData })
                    }
                  >
                    <Button
                      size="small"
                      icon={resolvedPrimary.icon}
                      data-cy={`entity-action-${resolvedPrimary.key}`}
                    >
                      {resolvedPrimary.label}
                    </Button>
                  </Popconfirm>
                ) : (
                  <Button
                    size="small"
                    icon={resolvedPrimary.icon}
                    data-cy={`entity-action-${resolvedPrimary.key}`}
                    onClick={() => resolvedPrimary.handler({ entity: record, refresh: fetchData })}
                  >
                    {resolvedPrimary.label}
                  </Button>
                )}
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
                          <span data-cy={`entity-action-${a.key}`}>{a.label}</span>
                        </Popconfirm>
                      ) : (
                        <span
                          data-cy={`entity-action-${a.key}`}
                          onClick={() => a.handler({ entity: record, refresh: fetchData })}
                        >
                          {a.label}
                        </span>
                      ),
                      icon: a.icon,
                    })),
                  }}
                  placement="bottomRight"
                >
                  <Button data-cy="entity-action-dropdown" size="small" icon={<MoreOutlined />} />
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
        listMode={listMode}
      />

      {/* <TagSummary
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
      /> */}

      {viewMode === 'grid' && renderGridItem ? (
        <div>
          {!loading && items.length === 0 ? (
            <div className="py-16 flex items-center justify-center">{renderEmptyState()}</div>
          ) : (
            <>
              <div className="grid gap-4 grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4">
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
                              a.handler({
                                entity: item,
                                refresh: fetchData,
                                selected: selectedRowKeys,
                              });
                            }}
                            onCancel={(e) => {
                              e?.stopPropagation?.();
                            }}
                          >
                            <Button
                              icon={a.icon}
                              type="text"
                              onClick={(e) => e.stopPropagation()}
                            />
                          </Popconfirm>
                        ) : (
                          <Button
                            icon={a.icon}
                            type="text"
                            onClick={(e) => {
                              e.stopPropagation();
                              a.handler({
                                entity: item,
                                refresh: fetchData,
                                selected: selectedRowKeys,
                              });
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

              {/* Pagination */}
              {pagination.total > pagination.pageSize && (
                <div className="mt-6 flex justify-between items-center pb-4">
                  <Button
                    onClick={() => goToPage(pagination.current - 1)}
                    disabled={pagination.current === 1}
                  >
                    Previous
                  </Button>
                  <span className="text-sm text-gray-500">
                    Page {pagination.current} of {Math.ceil(pagination.total / pagination.pageSize)}
                  </span>
                  <Button
                    onClick={() => goToPage(pagination.current + 1)}
                    disabled={pagination.current * pagination.pageSize >= pagination.total}
                  >
                    Next
                  </Button>
                </div>
              )}
            </>
          )}
        </div>
      ) : listMode && renderListItem ? (
        <div className="mt-4">
          <List
            itemLayout="vertical"
            dataSource={items}
            renderItem={renderListItem}
            bordered
            locale={{ emptyText: renderEmptyState() }}
            className="overflow-hidden bg-white dark:bg-gray-950 !border-gray-200 dark:!border-gray-800"
          />
          {items.length < pagination.total! && (
            <div className="flex justify-between items-center mt-4">
              <Button
                onClick={() => goToPage(pagination.current - 1)}
                disabled={pagination.current === 1}
              >
                Previous
              </Button>
              <span className="text-sm text-gray-500">
                Page {pagination.current} of{' '}
                {Math.ceil((pagination.total ?? 0) / pagination.pageSize)}
              </span>
              <Button
                onClick={() => goToPage(pagination.current + 1)}
                disabled={pagination.current * pagination.pageSize >= (pagination.total ?? 0)}
              >
                Next
              </Button>
            </div>
          )}
        </div>
      ) : (
        <div id="scrollable-entity-table" className="h-full flex flex-col overflow-hidden">
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
            scroll={{ y: scrollHeight }}
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
            onRow={(record) => ({
              onClick: () => onRowClick?.(record),
              'data-cy': `entity-${getRowKey(record)}`,
            })}
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
            className="bg-white dark:bg-gray-900 border-1 border-gray-200 dark:border-gray-800 rounded-lg"
          />
        </div>
      )}
    </div>
  );
}) as <T>(props: EntityListProps<T> & { ref?: React.Ref<EntityListHandle> }) => JSX.Element;

export { EntityList };
