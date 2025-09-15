import { Table, Empty, Button, Dropdown, Popconfirm, Tooltip, List } from 'antd';
import { ReloadOutlined, MoreOutlined } from '@ant-design/icons';
import { forwardRef, useEffect, useImperativeHandle, useRef, useState, type JSX } from 'react';
import type { ColumnsType } from 'antd/es/table';
import type { SortOption } from '@/types/common';

import { useNotifier } from '@/components/common';
import { useEntityViewState } from '@/hooks/useEntityViewState';
import type { ModuleRole } from '@/types/modules';
import ControlBar from './ControlBar';
import { GridSkeleton, RowsSkeleton } from '@/EntitySkeletons';

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
  emptyNoEntities?: React.ReactNode;
  showControlBar?: boolean;
};

export type EntityListHandle = {
  refresh: () => void;
  clearSelection: () => void;
  getSelectedRowKeys: () => React.Key[];
  /** Optimistically update a single row by key with a partial patch */
  updateRow: (key: React.Key, patch: Partial<any>) => void;
  /** Remove a set of rows by keys (keeps pagination.total in sync) */
  removeRows: (keys: React.Key[]) => void;
  /** Insert or update rows; mode=replace replaces by key if exists, else inserts (append/prepend) */
  upsertRows: (rows: any[], mode?: 'append' | 'prepend' | 'replace') => void;
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
    emptyNoEntities,
    showControlBar = true,
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
  const [loading, setLoading] = useState(true);
  const [items, setItems] = useState<T[] | null>(null); // null => initial not-fetched

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

  const fetchSeq = useRef(0);

  /** Fetch helpers
   * - Skeleton only when items === null (first load)
   * - On subsequent loads, keep old content; just flip loading
   */
  const fetchData = async () => {
    const seq = ++fetchSeq.current;
    const firstLoad = items === null;

    if (firstLoad) {
      // show skeleton area while initial fetch happens
      setItems(null);
    }
    setLoading(true);

    try {
      const res = await fetchItems({
        page: pagination.current,
        per_page: pagination.pageSize,
        query: searchTerm,
        sort: sorterState,
        filters: filterState,
      });
      if (fetchSeq.current !== seq) return;
      setItems(res.items);
      setPagination({ total: res.total });
    } catch (e) {
      if (fetchSeq.current !== seq) return;
      // If the very first fetch fails, allow the empty-gate to render
      if (items === null) setItems([]);
      notifyError('Fetch Failed', 'Could not load data');
    } finally {
      if (fetchSeq.current === seq) setLoading(false);
    }
  };

  const goToPage = async (page: number) => {
    const seq = ++fetchSeq.current;
    const firstLoad = items === null;

    if (firstLoad) {
      setItems(null);
    }
    setLoading(true);

    try {
      const res = await fetchItems({
        page,
        per_page: pagination.pageSize,
        query: searchTerm,
        sort: sorterState,
        filters: filterState,
      });
      if (fetchSeq.current !== seq) return;
      setItems(res.items);
      setPagination({ current: page, total: res.total });
    } catch {
      if (items === null) setItems([]);
      notifyError('Fetch Failed', 'Could not fetch data');
    } finally {
      if (fetchSeq.current === seq) setLoading(false);
    }
  };

  // Key resolver (stable reference)
  const keyOf = (item: T) => props.getRowKey(item);

  // --- Local mutation helpers ---
  const updateRow = (key: React.Key, patch: Partial<T>) => {
    setItems((prev) => {
      if (!prev) return prev;
      let changed = false;
      const next = prev.map((it) => {
        if (keyOf(it) === key) {
          changed = true;
          return { ...it, ...patch };
        }
        return it;
      });
      return changed ? next : prev;
    });
  };

  const removeRows = (keys: React.Key[]) => {
    if (!keys.length) return;

    setItems((prev) => {
      if (!prev) return prev;

      const keySet = new Set(keys);
      const next = prev.filter((it) => !keySet.has(keyOf(it)));

      // keep pagination.total in sync if it exists
      if (next.length !== prev.length) {
        const removedCount = prev.length - next.length;

        // use the *current* pagination from closure; pass a partial object (no function)
        const nextTotal = Math.max(0, (pagination.total ?? prev.length) - removedCount);
        const nextCurrent =
          next.length === 0 && (pagination.current ?? 1) > 1
            ? (pagination.current as number) - 1
            : pagination.current;

        setPagination({
          total: nextTotal,
          current: nextCurrent,
        });
      }

      return next;
    });

    // clear selection of removed rows (non-functional form)
    setSelectedRowKeys(selectedRowKeys.filter((k) => !keys.includes(k)));
  };

  const upsertRows = (rows: T[], mode: 'append' | 'prepend' | 'replace' = 'replace') => {
    if (!rows.length) return;
    setItems((prev) => {
      const prevList = prev ?? [];
      const byKey = new Map(prevList.map((r) => [keyOf(r), r]));
      for (const r of rows) {
        const k = keyOf(r);
        if (byKey.has(k)) {
          // replace existing
          byKey.set(k, { ...byKey.get(k)!, ...r });
        } else {
          // insert new
          if (mode === 'prepend') {
            byKey.set(Symbol('___prepend___') as any, r); // marker to control order
          } else {
            byKey.set(k, r);
          }
        }
      }
      // rebuild preserving original order + new inserts at chosen side
      const existing = prevList.map((r) => byKey.get(keyOf(r))!).filter(Boolean);
      const inserted = rows.filter((r) => !prevList.some((p) => keyOf(p) === keyOf(r)));
      const next =
        mode === 'prepend'
          ? [...inserted, ...existing]
          : mode === 'append'
            ? [...existing, ...inserted]
            : existing; // replace mode doesn't add brand-new rows unless they were in rows
      return next;
    });
  };

  // --- expose in imperative handle ---
  useImperativeHandle(ref, () => ({
    refresh: fetchData,
    clearSelection: () => setSelectedRowKeys([]),
    getSelectedRowKeys: () => selectedRowKeys,
    updateRow,
    removeRows,
    upsertRows,
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

  // Reusable filtered empty-state (table-style)
  const renderFilteredEmptyState = () => (
    <div className="flex justify-center w-full py-8 sm:py-12 rounded-xl border-2 border-dashed bg-white dark:bg-gray-900 border-gray-200 dark:border-gray-800 text-center">
      <Empty image={Empty.PRESENTED_IMAGE_SIMPLE} description="No results found.">
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
    </div>
  );

  const isPristine =
    !searchTerm.trim() && sorterState.length === 0 && Object.keys(filterState).length === 0;

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
      className: 'whitespace-nowrap',
      onCell: () => ({ style: { width: '1%', whiteSpace: 'nowrap' } }),
      render: (_, record) => {
        const entityActions = actions.entity!(record);
        if (!entityActions.length) return null;

        const primary = entityActions.find((a) => a.isPrimary) ?? entityActions[0];
        const secondary = entityActions.filter((a) => a.key !== primary.key);

        const run = (a: (typeof entityActions)[number]) =>
          a.handler({ entity: record, refresh: fetchData });

        const menuItems = secondary.map((a) =>
          a.confirm
            ? {
                key: a.key,
                icon: a.icon,
                label: (
                  <Popconfirm
                    title={`Are you sure you want to ${a.label.toLowerCase()}?`}
                    okText="Yes"
                    cancelText="No"
                    okButtonProps={{
                      'data-testid': 'confirm-yes',
                      danger: a.key === 'delete',
                    }}
                    cancelButtonProps={{ 'data-testid': 'confirm-no' }}
                    onConfirm={() => run(a)}
                  >
                    <span
                      data-testid={`entity-action-${a.key}`}
                      className="block -mx-3 -my-1.5 !px-3 !py-1.5"
                      onMouseDown={(e) => {
                        e.preventDefault();
                        e.stopPropagation();
                      }}
                    >
                      {a.label}
                    </span>
                  </Popconfirm>
                ),
              }
            : {
                key: a.key,
                icon: a.icon,
                label: (
                  <span
                    className="block -mx-3 -my-1.5 !px-3 !py-1.5"
                    data-testid={`entity-action-${a.key}`}
                  >
                    {a.label}
                  </span>
                ),
                onClick: ({ domEvent }: any) => {
                  domEvent.stopPropagation();
                  run(a);
                },
              },
        );

        return (
          <div
            onClick={(e) => e.stopPropagation()}
            data-testid="entity-actions"
            className="flex justify-end"
          >
            {secondary.length === 0 ? (
              primary.confirm ? (
                <Popconfirm
                  title={`Are you sure you want to ${primary.label.toLowerCase()}?`}
                  okText="Yes"
                  cancelText="No"
                  okButtonProps={{ 'data-testid': 'confirm-yes' }}
                  cancelButtonProps={{ 'data-testid': 'confirm-no' }}
                  onConfirm={() => run(primary)}
                >
                  <Button
                    size="small"
                    icon={primary.icon}
                    className="!px-2"
                    data-testid={`entity-action-${primary.key}`}
                  >
                    <span className="truncate max-w-[140px] inline-block">{primary.label}</span>
                  </Button>
                </Popconfirm>
              ) : (
                <Button
                  size="small"
                  icon={primary.icon}
                  className="!px-2"
                  data-testid={`entity-action-${primary.key}`}
                  onClick={() => run(primary)}
                >
                  <span className="truncate max-w-[140px] inline-block">{primary.label}</span>
                </Button>
              )
            ) : (
              <div className="flex items-center gap-1">
                {primary.confirm ? (
                  <Popconfirm
                    title={`Are you sure you want to ${primary.label.toLowerCase()}?`}
                    okText="Yes"
                    cancelText="No"
                    okButtonProps={{ 'data-testid': 'confirm-yes' }}
                    cancelButtonProps={{ 'data-testid': 'confirm-no' }}
                    onConfirm={() => run(primary)}
                  >
                    <Button
                      size="small"
                      icon={primary.icon}
                      className="!px-2"
                      data-testid={`entity-action-${primary.key}`}
                    >
                      <span className="truncate max-w-[140px] inline-block">{primary.label}</span>
                    </Button>
                  </Popconfirm>
                ) : (
                  <Button
                    size="small"
                    icon={primary.icon}
                    className="!px-2"
                    data-testid={`entity-action-${primary.key}`}
                    onClick={() => run(primary)}
                  >
                    <span className="truncate max-w-[140px] inline-block">{primary.label}</span>
                  </Button>
                )}

                <Dropdown trigger={['click']} placement="bottomRight" menu={{ items: menuItems }}>
                  <Button
                    data-testid="entity-action-dropdown"
                    size="small"
                    icon={<MoreOutlined />}
                  />
                </Dropdown>
              </div>
            )}
          </div>
        );
      },
    });
  }

  const isInitialLoading = items === null;
  const isEmpty = !isInitialLoading && !loading && items!.length === 0;

  // ---- Derived UI state (keeps ControlBar mounted) ----
  const hasRefinements = hasSearch || hasSort || hasFilters;
  const shouldShowControlBar = showControlBar && (isInitialLoading || !isEmpty || hasRefinements);

  return (
    <div className="h-full flex flex-col">
      {shouldShowControlBar && (
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
      )}

      <div className="flex-1 min-h-0">
        {isInitialLoading ? (
          // Initial load → skeletons (ControlBar remains mounted → no focus loss)
          viewMode === 'grid' && renderGridItem ? (
            <GridSkeleton count={8} />
          ) : (
            <RowsSkeleton rows={6} />
          )
        ) : isEmpty ? (
          // Global empty gate
          <div className="w-full h-full min-h-0 flex">
            <div className="flex-1">
              {isPristine && !!emptyNoEntities ? emptyNoEntities : renderFilteredEmptyState()}
            </div>
          </div>
        ) : (
          // Actual content
          <>
            {viewMode === 'grid' && renderGridItem ? (
              <div>
                {!loading && items!.length === 0 ? (
                  <div className="flex items-center justify-center">
                    {renderFilteredEmptyState()}
                  </div>
                ) : (
                  <>
                    <div
                      className="grid gap-4 grid-cols-1 lg:grid-cols-2 xl:grid-cols-3 2xl:grid-cols-4"
                      data-testid="entity-grid"
                    >
                      {items!.map((item) => {
                        const allActions = actions?.entity?.(item) ?? [];
                        if (!allActions.length) {
                          return <div key={getRowKey(item)}>{renderGridItem(item, [])}</div>;
                        }

                        const swallow = (e: React.MouseEvent) => {
                          e.preventDefault();
                          e.stopPropagation();
                        };

                        const primary = allActions.find((a) => a.isPrimary) ?? allActions[0];
                        const secondary = allActions.filter((a) => a.key !== primary.key);

                        const InlineButton = (
                          <Tooltip title={primary.label} key={primary.key}>
                            {primary.confirm ? (
                              <Popconfirm
                                title={`Are you sure you want to ${primary.label.toLowerCase()}?`}
                                okText="Yes"
                                cancelText="No"
                                okButtonProps={{ 'data-testid': 'confirm-yes' }}
                                cancelButtonProps={{ 'data-testid': 'confirm-no' }}
                                onConfirm={(e) => {
                                  e?.preventDefault?.();
                                  e?.stopPropagation?.();
                                  primary.handler({
                                    entity: item,
                                    refresh: fetchData,
                                    selected: selectedRowKeys,
                                  });
                                }}
                                onCancel={(e) => {
                                  e?.preventDefault?.();
                                  e?.stopPropagation?.();
                                }}
                              >
                                <Button
                                  icon={primary.icon}
                                  type="text"
                                  data-testid={`entity-action-${primary.key}`}
                                  onClick={swallow}
                                />
                              </Popconfirm>
                            ) : (
                              <Button
                                icon={primary.icon}
                                type="text"
                                data-testid={`entity-action-${primary.key}`}
                                onClick={(e) => {
                                  swallow(e);
                                  primary.handler({
                                    entity: item,
                                    refresh: fetchData,
                                    selected: selectedRowKeys,
                                  });
                                }}
                              />
                            )}
                          </Tooltip>
                        );

                        const DropdownButton =
                          secondary.length > 0 ? (
                            <Dropdown
                              key="more"
                              menu={{
                                items: secondary.map((a) => ({
                                  key: a.key,
                                  label: a.confirm ? (
                                    <Popconfirm
                                      title={`Are you sure you want to ${a.label.toLowerCase()}?`}
                                      okText="Yes"
                                      cancelText="No"
                                      okButtonProps={{ 'data-testid': 'confirm-yes' }}
                                      cancelButtonProps={{ 'data-testid': 'confirm-no' }}
                                      onConfirm={(e) => {
                                        e?.preventDefault?.();
                                        e?.stopPropagation?.();
                                        a.handler({
                                          entity: item,
                                          refresh: fetchData,
                                          selected: selectedRowKeys,
                                        });
                                      }}
                                      onCancel={(e) => {
                                        e?.preventDefault?.();
                                        e?.stopPropagation?.();
                                      }}
                                    >
                                      <span
                                        data-testid={`entity-action-${a.key}`}
                                        onClick={swallow}
                                      >
                                        {a.label}
                                      </span>
                                    </Popconfirm>
                                  ) : (
                                    <span
                                      onClick={(e) => {
                                        swallow(e);
                                        a.handler({
                                          entity: item,
                                          refresh: fetchData,
                                          selected: selectedRowKeys,
                                        });
                                      }}
                                      data-testid={`entity-action-${a.key}`}
                                    >
                                      {a.label}
                                    </span>
                                  ),
                                  icon: a.icon,
                                })),
                              }}
                              placement="bottomRight"
                            >
                              <Button
                                type="text"
                                icon={<MoreOutlined />}
                                data-testid="entity-action-dropdown"
                                onClick={swallow}
                              />
                            </Dropdown>
                          ) : null;

                        const actionButtons = [InlineButton, DropdownButton].filter(Boolean);
                        return (
                          <div key={getRowKey(item)}>{renderGridItem(item, actionButtons)}</div>
                        );
                      })}
                    </div>

                    {pagination.total > pagination.pageSize && (
                      <div className="mt-6 flex justify-between items-center pb-4">
                        <Button
                          onClick={() => goToPage(pagination.current - 1)}
                          disabled={pagination.current === 1}
                          data-testid="grid-previous"
                        >
                          Previous
                        </Button>
                        <span className="text-sm text-gray-500">
                          Page {pagination.current} of{' '}
                          {Math.ceil(pagination.total / pagination.pageSize)}
                        </span>
                        <Button
                          onClick={() => goToPage(pagination.current + 1)}
                          disabled={pagination.current * pagination.pageSize >= pagination.total}
                          data-testid="grid-next"
                        >
                          Next
                        </Button>
                      </div>
                    )}
                  </>
                )}
              </div>
            ) : listMode && renderListItem ? (
              <>
                <List
                  itemLayout="vertical"
                  dataSource={items!}
                  renderItem={renderListItem}
                  bordered
                  locale={{ emptyText: renderFilteredEmptyState() }}
                  className="overflow-hidden bg-white dark:bg-gray-950 !border-gray-200 dark:!border-gray-800"
                  data-testid="entity-list"
                />
                {items!.length < (pagination.total ?? 0) && (
                  <div className="flex justify-between items-center mt-4">
                    <Button
                      onClick={() => goToPage(pagination.current - 1)}
                      disabled={pagination.current === 1}
                      data-testid="list-previous"
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
                      data-testid="list-next"
                    >
                      Next
                    </Button>
                  </div>
                )}
              </>
            ) : (
              <div id="scrollable-entity-table" className="h-full flex flex-col overflow-hidden">
                <Table<T>
                  columns={extendedColumns}
                  dataSource={items!}
                  rowKey={getRowKey}
                  loading={loading}
                  tableLayout="auto"
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
                  onChange={(pagination_, filters, sorter) => {
                    const sorterArray = (Array.isArray(sorter) ? sorter : [sorter])
                      .filter(
                        (s): s is { columnKey: string; order: 'ascend' | 'descend' } =>
                          !!s.columnKey && !!s.order,
                      )
                      .map((s) => ({ field: String(s.columnKey), order: s.order }));
                    setSorterState(sorterArray);
                    setFilterState(filters as Record<string, string[]>);
                    setPagination({
                      current: pagination_.current || 1,
                      pageSize: pagination_.pageSize || 10,
                    });
                  }}
                  onRow={(record) => ({
                    onClick: () => onRowClick?.(record),
                    'data-testid': 'entity-row',
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
                  data-testid="entity-table"
                  className="bg-white dark:bg-gray-900 border-1 border-gray-200 dark:border-gray-800 rounded-lg overflow-hidden"
                />
              </div>
            )}
          </>
        )}
      </div>
    </div>
  );
}) as <T>(props: EntityListProps<T> & { ref?: React.Ref<EntityListHandle> }) => JSX.Element;

export { EntityList };
