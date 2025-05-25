import { Table, Space, Button, Input, Popconfirm, Empty, Tooltip, Card, Statistic } from 'antd';
import {
  EditOutlined,
  DeleteOutlined,
  CheckOutlined,
  CloseOutlined,
  EyeOutlined,
  ReloadOutlined,
} from '@ant-design/icons';
import { useEffect, useState } from 'react';
import { useNavigate } from 'react-router-dom';
import type { ColumnsType } from 'antd/es/table';
import type { FilterDropdownProps } from 'antd/es/table/interface';
import type { Module } from '@/types/modules';
import { useTableQuery } from '@/hooks/useTableQuery';
import type { SortOption } from '@/types/common';
import { ModulesService } from '@/services/modules/mock'; // TODO: Remeber to change this when services work
import AppLayout from '@/layouts/AppLayout';
import TableControlBar from '@/components/TableControlBar';
import TableTagSummary from '@/components/TableTagSummary';
import TableCreateModal from '@/components/TableCreateModal';
import { useNotifier } from '@/components/Notifier';

export default function ModuleList() {
  // ======================================================================
  // =========================== State & Hooks ============================
  // ======================================================================

  const navigate = useNavigate();
  const { notifySuccess, notifyError } = useNotifier();

  const {
    searchTerm,
    setSearchTerm,
    sorterState,
    setSorterState,
    filterState,
    setFilterState,
    pagination,
    setPagination,
    clearSearch,
    clearSorters,
    clearFilters,
    clearAll,
  } = useTableQuery();

  const [modules, setModules] = useState<Module[]>([]);
  const [loading, setLoading] = useState(false);
  const [editingRowId, setEditingRowId] = useState<number | null>(null);
  const [editCache, setEditCache] = useState<Partial<Module>>({});
  const [selectedRowKeys, setSelectedRowKeys] = useState<React.Key[]>([]);
  const [isAddModalOpen, setIsAddModalOpen] = useState(false);
  const [newModule, setNewModule] = useState<Partial<Module>>({
    code: '',
    year: new Date().getFullYear(),
    description: '',
    credits: 16,
  });

  // ======================================================================
  // ============================ Fetch Modules ===========================
  // ======================================================================

  const fetchModules = async () => {
    setLoading(true);
    const sort: SortOption[] = sorterState.map(({ field, order }) => ({ field, order }));

    const res = await ModulesService.listModules({
      page: pagination.current,
      per_page: pagination.pageSize,
      query: searchTerm || undefined,
      code: filterState.code?.[0],
      year: filterState.year?.[0] ? parseInt(filterState.year[0]) : undefined,
      sort,
    });

    if (res.success) {
      setModules(res.data.modules);
      setPagination({ total: res.data.total });
    } else {
      notifyError('Failed to fetch modules');
    }

    setLoading(false);
  };

  useEffect(() => {
    fetchModules();
  }, [searchTerm, filterState, sorterState, pagination.current, pagination.pageSize]);

  // ======================================================================
  // ============================== Handlers ==============================
  // ======================================================================

  const handleAddModule = () => {
    setNewModule({
      code: '',
      year: new Date().getFullYear(),
      description: '',
      credits: 16,
    });
    setIsAddModalOpen(true);
  };

  const handleSubmitNewModule = async (values: Partial<Module>) => {
    const payload = {
      code: values.code ?? '',
      year: Number(values.year),
      description: values.description ?? '',
      credits: Number(values.credits),
    };

    const res = await ModulesService.createModule(payload);
    if (res.success) {
      notifySuccess('Module created successfully');
      setIsAddModalOpen(false);
      fetchModules();
    } else {
      notifyError('Failed to create module');
    }
  };

  const handleEditSave = async () => {
    if (!editingRowId || !editCache.code || !editCache.year || !editCache.description) return;

    const payload = {
      code: editCache.code,
      year: editCache.year,
      description: editCache.description,
      credits: editCache.credits || 0,
    };

    const res = await ModulesService.editModule(editingRowId, payload);
    if (res.success) {
      notifySuccess('Module updated successfully');
      setEditingRowId(null);
      setEditCache({});
      fetchModules();
    } else {
      notifyError('Failed to update module');
    }
  };

  const handleDelete = async (id: number) => {
    const res = await ModulesService.deleteModule(id);
    if (res.success) {
      notifySuccess('Module deleted');
      fetchModules();
    } else {
      notifyError('Delete failed');
    }
  };

  const handleBulkDelete = async () => {
    for (const id of selectedRowKeys) {
      await handleDelete(Number(id));
    }
    setSelectedRowKeys([]);
  };

  // ======================================================================
  // =========================== Table Columns ============================
  // ======================================================================

  const columns: ColumnsType<Module> = [
    {
      title: 'Code',
      dataIndex: 'code',
      key: 'code',
      sorter: { multiple: 1 },
      sortOrder: sorterState.find((s) => s.field === 'code')?.order ?? null,
      filteredValue: filterState.code || null,
      onFilter: () => true,
      filterDropdown: (props: FilterDropdownProps) => {
        const { setSelectedKeys, selectedKeys, confirm, clearFilters } = props;
        return (
          <div className="flex flex-col gap-2 p-2 w-56">
            <Input
              placeholder="Search code"
              value={selectedKeys[0]}
              onChange={(e) => setSelectedKeys([e.target.value])}
              onPressEnter={() => confirm()}
            />
            <div className="flex justify-between gap-2">
              <Button type="primary" size="small" onClick={() => confirm()}>
                Filter
              </Button>
              <Button
                size="small"
                onClick={() => {
                  clearFilters?.();
                  (props as any).closeDropdown?.();
                }}
              >
                Reset
              </Button>
            </div>
          </div>
        );
      },
      render: (_, record) =>
        editingRowId === record.id ? (
          <Input
            value={editCache.code}
            onChange={(e) => setEditCache((prev) => ({ ...prev, code: e.target.value }))}
          />
        ) : (
          record.code.replace(/([A-Za-z]+)(\d+)/, '$1 $2')
        ),
    },
    {
      title: 'Year',
      dataIndex: 'year',
      key: 'year',
      sorter: { multiple: 2 },
      sortOrder: sorterState.find((s) => s.field === 'year')?.order ?? null,
      filteredValue: filterState.year || null,
      onFilter: () => true,
      filterDropdown: (props: FilterDropdownProps) => {
        const { setSelectedKeys, selectedKeys, confirm, clearFilters } = props;
        return (
          <div className="flex flex-col gap-2 p-2 w-56">
            <Input
              placeholder="Search year"
              value={selectedKeys[0]}
              onChange={(e) => setSelectedKeys([e.target.value])}
              onPressEnter={() => confirm()}
            />
            <div className="flex justify-between gap-2">
              <Button type="primary" size="small" onClick={() => confirm()}>
                Filter
              </Button>
              <Button
                size="small"
                onClick={() => {
                  clearFilters?.();
                  (props as any).closeDropdown?.();
                }}
              >
                Reset
              </Button>
            </div>
          </div>
        );
      },
      render: (_, record) =>
        editingRowId === record.id ? (
          <Input
            type="number"
            value={editCache.year}
            onChange={(e) => setEditCache((prev) => ({ ...prev, year: parseInt(e.target.value) }))}
          />
        ) : (
          record.year
        ),
    },
    {
      title: 'Description',
      dataIndex: 'description',
      key: 'description',
      sorter: { multiple: 3 },
      sortOrder: sorterState.find((s) => s.field === 'description')?.order ?? null,
      render: (_, record) =>
        editingRowId === record.id ? (
          <Input
            value={editCache.description}
            onChange={(e) => setEditCache((prev) => ({ ...prev, description: e.target.value }))}
          />
        ) : (
          record.description
        ),
    },
    {
      title: 'Credits',
      dataIndex: 'credits',
      key: 'credits',
      sorter: { multiple: 4 },
      sortOrder: sorterState.find((s) => s.field === 'credits')?.order ?? null,
      render: (_, record) =>
        editingRowId === record.id ? (
          <Input
            type="number"
            value={editCache.credits}
            onChange={(e) =>
              setEditCache((prev) => ({ ...prev, credits: parseInt(e.target.value) }))
            }
          />
        ) : (
          record.credits
        ),
    },
    {
      title: 'Actions',
      key: 'actions',
      align: 'right',
      width: 120,
      render: (_, record) =>
        editingRowId === record.id ? (
          <Space>
            <Tooltip title="Save">
              <Button
                type="primary"
                shape="circle"
                icon={<CheckOutlined />}
                size="small"
                onClick={handleEditSave}
              />
            </Tooltip>
            <Tooltip title="Cancel">
              <Button
                shape="circle"
                icon={<CloseOutlined />}
                size="small"
                onClick={() => setEditingRowId(null)}
              />
            </Tooltip>
          </Space>
        ) : (
          <Space>
            <Tooltip title="View module">
              <Button
                type="text"
                icon={<EyeOutlined />}
                onClick={() => navigate(`/modules/${record.id}`)}
                size="small"
              />
            </Tooltip>
            <Tooltip title="Edit">
              <Button
                type="text"
                icon={<EditOutlined />}
                onClick={() => {
                  setEditingRowId(record.id);
                  setEditCache(record);
                }}
                size="small"
              />
            </Tooltip>
            <Tooltip title="Delete">
              <Popconfirm
                title="Delete this module?"
                onConfirm={() => handleDelete(record.id)}
                okText="Yes"
                cancelText="No"
              >
                <Button type="text" icon={<DeleteOutlined />} danger size="small" />
              </Popconfirm>
            </Tooltip>
          </Space>
        ),
    },
  ];

  // ======================================================================
  // ============================== Render ================================
  // ======================================================================

  return (
    <AppLayout title="Modules" description="All the modules in the COS department">
      <div className="mb-6 flex flex-wrap gap-4">
        <Card className="flex-1 min-w-[200px]">
          <Statistic title="Total Modules" value={modules.length} />
        </Card>
      </div>

      <TableControlBar
        handleSearch={setSearchTerm}
        searchTerm={searchTerm}
        handleAdd={handleAddModule}
        addButtonText="Add Module"
        handleBulkDelete={handleBulkDelete}
        selectedRowKeys={selectedRowKeys}
        clearMenuItems={[
          { key: 'clear-search', label: 'Clear Search', onClick: clearSearch },
          { key: 'clear-sort', label: 'Clear Sort', onClick: clearSorters },
          { key: 'clear-filters', label: 'Clear Filters', onClick: clearFilters },
          { key: 'clear-all', label: 'Clear All', onClick: clearAll },
        ]}
        searchPlaceholder="Search code or description"
        bulkDeleteConfirmMessage="Delete selected modules?"
      />

      <TableTagSummary
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

      <Table<Module>
        rowKey="id"
        columns={columns}
        dataSource={modules}
        rowSelection={{
          selectedRowKeys,
          onChange: setSelectedRowKeys,
        }}
        loading={loading}
        pagination={{
          ...pagination,
          showSizeChanger: true,
          showQuickJumper: true,
          onChange: (page, pageSize) => setPagination({ current: page, pageSize }),
        }}
        onChange={(pagination, filters, sorter) => {
          const sorterArray = (Array.isArray(sorter) ? sorter : [sorter])
            .filter(
              (s): s is { columnKey: string; order: 'ascend' | 'descend' } =>
                !!s.columnKey && !!s.order,
            )
            .map((s) => ({
              field: String(s.columnKey),
              order: s.order,
            }));

          setSorterState(sorterArray);
          setFilterState(filters as Record<string, string[]>);
          setPagination({
            current: pagination.current ?? 1,
            pageSize: pagination.pageSize ?? 10,
          });
        }}
        locale={{
          emptyText: (
            <Empty image={Empty.PRESENTED_IMAGE_SIMPLE} description="No modules found.">
              <Button icon={<ReloadOutlined />} onClick={clearAll}>
                Clear All Filters
              </Button>
            </Empty>
          ),
        }}
        className="border-1 border-gray-200 dark:border-gray-800 rounded-lg overflow-hidden"
      />

      <TableCreateModal
        open={isAddModalOpen}
        onCancel={() => setIsAddModalOpen(false)}
        onCreate={handleSubmitNewModule}
        title="Add Module"
        initialValues={newModule}
        onChange={(values) => setNewModule(values)}
        fields={[
          { name: 'code', label: 'Module Code', type: 'text', required: true },
          { name: 'year', label: 'Year', type: 'number', required: true },
          { name: 'description', label: 'Description', type: 'text', required: true },
          { name: 'credits', label: 'Credits', type: 'number', required: true },
        ]}
      />
    </AppLayout>
  );
}
