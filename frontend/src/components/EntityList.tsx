import { Table, Empty, Button, Popconfirm, Tooltip, Dropdown, Pagination } from 'antd';
import {
  DeleteOutlined,
  EditOutlined,
  EyeOutlined,
  MoreOutlined,
  ReloadOutlined,
} from '@ant-design/icons';
import { useEffect, useState } from 'react';
import type { ColumnsType } from 'antd/es/table';
import type { ItemType } from 'antd/es/menu/interface';
import type { SortOption } from '@/types/common';

import ControlBar from '@/components/ControlBar';
import TagSummary from '@/components/TagSummary';
import CreateModal from '@/components/CreateModal';
import EditModal from '@/components/EditModal';
import { useNotifier } from '@/components/Notifier';
import { useEntityViewState } from '@/hooks/useEntityViewState';
import { PAGE_SIZE_OPTIONS } from '@/constants/pagination';

/**
 * Generic props for listing and managing entities.
 */
type EntityListProps<T> = {
  name: string;
  viewModeKey?: string;

  /** Fetch items based on query, filters, and pagination. */
  fetchItems: (params: {
    page: number;
    per_page: number;
    query: string;
    sort: SortOption[];
    filters: Record<string, string[]>;
  }) => Promise<{ items: T[]; total: number }>;

  columns: ColumnsType<T>;
  getRowKey: (item: T) => string | number;
  onRowClick?: (item: T) => void;

  /** Render function for grid view items. */
  renderGridItem?: (item: T, actions: React.ReactNode[]) => React.ReactNode;

  /** Modal for creating items. */
  createModal?: {
    title: string;
    fields: any[];
    onCreate: (values: any) => Promise<void>;
    getInitialValues: () => Partial<T>;
  };

  /** Modal for editing items. */
  editModal?: {
    title: string;
    fields: any[];
    onEdit: (item: T, values: any) => Promise<void>;
  };

  /** Optional delete handler. */
  onDelete?: (item: T) => Promise<void>;

  /** Sorting options for grid view. */
  sortOptions?: { label: string; field: string }[];

  /** Filtering groups for grid view. */
  filterGroups?: {
    key: string;
    label: string;
    type: 'text' | 'select';
    options?: { label: string; value: string }[];
  }[];

  actions?: React.ReactNode;
};

/**
 * A generic, reusable entity list component that supports
 * table and grid views, filtering, sorting, pagination,
 * inline and dropdown actions, and modals for create/edit.
 */
export function EntityList<T>(props: EntityListProps<T>) {
  const {
    name,
    fetchItems,
    viewModeKey = `${name.toLowerCase().replace(/\s+/g, '_')}_view_mode`,
    columns,
    getRowKey,
    onRowClick,
    renderGridItem,
    createModal,
    editModal,
    onDelete,
    sortOptions,
    filterGroups,
  } = props;

  // View state is managed via a custom hook
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
    clearAll,
    clearSearch,
    clearSorters,
    clearFilters,
    editModalOpen,
    setEditModalOpen,
    editingItem,
    setEditingItem,
    isAddModalOpen,
    setIsAddModalOpen,
    newItem,
    setNewItem,
  } = useEntityViewState<T>({
    viewModeKey,
    getInitialNewItem: () => createModal?.getInitialValues() ?? ({} as Partial<T>),
  });

  const { notifyError } = useNotifier();
  const [loading, setLoading] = useState(false);
  const [items, setItems] = useState<T[]>([]);

  /**
   * Fetches item list based on current state.
   */
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

  /**
   * Fetch immediately on search, filter, sort, or pagination changes.
   * No debounce – fetch runs instantly on every state change.
   */
  useEffect(() => {
    fetchData();
  }, [searchTerm, filterState, sorterState, pagination.current, pagination.pageSize]);

  /**
   * Deletes an item and refetches data.
   */
  const handleDelete = async (item: T) => {
    if (!onDelete) return;
    await onDelete(item);
    await fetchData();
  };

  /**
   * Inline icon-based actions for grid view.
   */
  const renderInlineActions = (item: T): React.ReactNode[] => {
    const key = getRowKey(item);
    return [
      onRowClick && (
        <Tooltip title="View" key={`${key}-view`}>
          <EyeOutlined
            onClick={(e) => {
              e.stopPropagation();
              onRowClick?.(item);
            }}
          />
        </Tooltip>
      ),
      editModal && (
        <Tooltip title="Edit" key={`${key}-edit`}>
          <EditOutlined
            onClick={(e) => {
              e.stopPropagation();
              setEditingItem(item);
              setEditModalOpen(true);
            }}
          />
        </Tooltip>
      ),
      onDelete && (
        <Tooltip title="Delete" key={`${key}-delete`}>
          <Popconfirm
            title="Delete this item?"
            onConfirm={(e) => {
              e?.stopPropagation();
              handleDelete(item);
            }}
            onCancel={(e) => e?.stopPropagation()}
            okText="Yes"
            cancelText="No"
          >
            <DeleteOutlined onClick={(e) => e.stopPropagation()} />
          </Popconfirm>
        </Tooltip>
      ),
    ].filter(Boolean);
  };

  /**
   * Adds an "Actions" column to the table if needed.
   */
  const extendedColumns: ColumnsType<T> =
    (editModal || onDelete || onRowClick) && !columns.some((col) => col.key === 'actions')
      ? [
          ...columns,
          {
            title: 'Actions',
            key: 'actions',
            align: 'right',
            width: 100,
            render: (_, record: any) => (
              <div onClick={(e) => e.stopPropagation()}>
                <Dropdown
                  trigger={['click']}
                  menu={{
                    items: [
                      onRowClick && {
                        key: 'view',
                        icon: <EyeOutlined />,
                        label: 'View',
                      },
                      editModal && {
                        key: 'edit',
                        icon: <EditOutlined />,
                        label: 'Edit',
                      },
                      onDelete && {
                        key: 'delete',
                        icon: <DeleteOutlined />,
                        danger: true,
                        label: (
                          <Popconfirm
                            title="Delete this item?"
                            onConfirm={(e) => {
                              e?.stopPropagation();
                              handleDelete(record);
                            }}
                            onCancel={(e) => e?.stopPropagation()}
                            okText="Yes"
                            cancelText="No"
                          >
                            <span onClick={(e) => e.stopPropagation()}>Delete</span>
                          </Popconfirm>
                        ),
                      },
                    ].filter(Boolean) as ItemType[],
                    onClick: ({ key, domEvent }) => {
                      domEvent.preventDefault();
                      domEvent.stopPropagation();

                      if (key === 'view') onRowClick?.(record);
                      else if (key === 'edit') {
                        setEditingItem(record);
                        setEditModalOpen(true);
                      }
                    },
                  }}
                >
                  <Button icon={<MoreOutlined />} style={{ borderRadius: 6 }} />
                </Dropdown>
              </div>
            ),
          },
        ]
      : columns;

  return (
    <div>
      {/* Control bar with search, view mode, add, sort, filters */}
      <ControlBar
        handleSearch={setSearchTerm}
        searchTerm={searchTerm}
        viewMode={viewMode}
        onViewModeChange={setViewMode}
        handleAdd={createModal ? () => setIsAddModalOpen(true) : undefined}
        addButtonText={createModal ? `Add ${name.slice(0, -1)}` : undefined}
        selectedRowKeys={selectedRowKeys}
        clearMenuItems={[
          { key: 'clear-search', label: 'Clear Search', onClick: clearSearch },
          { key: 'clear-sort', label: 'Clear Sort', onClick: clearSorters },
          { key: 'clear-filters', label: 'Clear Filters', onClick: clearFilters },
          { key: 'clear-all', label: 'Clear All', onClick: clearAll },
        ]}
        searchPlaceholder={`Search ${name.toLowerCase()}`}
        sortOptions={viewMode === 'grid' ? sortOptions : undefined}
        currentSort={sorterState.map((s) => `${s.field}.${s.order}`)}
        onSortChange={(vals) => {
          setSorterState(
            vals.map((v) => {
              const [field, order] = v.split('.');
              return { field, order: order as 'ascend' | 'descend' };
            }),
          );
        }}
        filterGroups={viewMode === 'grid' ? filterGroups : undefined}
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
        actions={props.actions}
      />

      {/* Current filters and sort tags */}
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

      {/* Table View */}
      {viewMode === 'table' ? (
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
          rowSelection={{
            selectedRowKeys,
            onChange: setSelectedRowKeys,
          }}
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
                <Button icon={<ReloadOutlined />} onClick={clearAll}>
                  Clear All Filters
                </Button>
              </Empty>
            ),
          }}
          className="bg-white dark:bg-gray-900 border-1 border-gray-200 dark:border-gray-800 rounded-lg overflow-hidden"
        />
      ) : items.length === 0 ? (
        <Empty image={Empty.PRESENTED_IMAGE_SIMPLE} description="No data found." className="mt-10">
          <Button icon={<ReloadOutlined />} onClick={clearAll}>
            Clear All Filters
          </Button>
        </Empty>
      ) : (
        <>
          {/* Grid View */}
          <div className="grid grid-cols-1 sm:grid-cols-2 md:grid-cols-3 lg:grid-cols-4 gap-4">
            {items.map((item) =>
              renderGridItem ? renderGridItem(item, renderInlineActions(item)) : null,
            )}
          </div>

          <div className="mt-6 flex justify-center items-center w-full">
            <Pagination
              current={pagination.current}
              pageSize={pagination.pageSize}
              total={pagination.total}
              showSizeChanger
              showQuickJumper
              pageSizeOptions={PAGE_SIZE_OPTIONS}
              onChange={(page, pageSize) => setPagination({ current: page, pageSize })}
            />
          </div>
        </>
      )}

      {/* Create Modal */}
      {createModal && (
        <CreateModal
          open={isAddModalOpen}
          onCancel={() => setIsAddModalOpen(false)}
          onCreate={async (values) => {
            await createModal.onCreate(values);
            await fetchData();
            setIsAddModalOpen(false);
          }}
          title={createModal.title}
          fields={createModal.fields}
          initialValues={newItem}
          onChange={(values) => setNewItem(values as Partial<T>)}
        />
      )}

      {/* Edit Modal */}
      {editModal && editingItem && (
        <EditModal
          open={editModalOpen}
          onCancel={() => setEditModalOpen(false)}
          onEdit={async (values) => {
            await editModal.onEdit(editingItem, values);
            await fetchData();
            setEditModalOpen(false);
            setEditingItem(null);
          }}
          title={editModal.title}
          fields={editModal.fields}
          initialValues={editingItem}
          onChange={(val) => setEditingItem({ ...editingItem, ...val })}
        />
      )}
    </div>
  );
}
