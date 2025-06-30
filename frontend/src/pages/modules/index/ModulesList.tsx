import { Table, Button, Input, Popconfirm, Empty, Tooltip, Select, Dropdown, Col, Row } from 'antd';
import {
  EditOutlined,
  DeleteOutlined,
  EyeOutlined,
  ReloadOutlined,
  MoreOutlined,
} from '@ant-design/icons';
import { useEffect, useState } from 'react';
import { useNavigate } from 'react-router-dom';
import type { ColumnsType } from 'antd/es/table';
import type { FilterDropdownProps } from 'antd/es/table/interface';
import type { Module } from '@/types/modules';
import { useTableQuery } from '@/hooks/useTableQuery';
import type { SortOption } from '@/types/common';
import ControlBar from '@/components/ControlBar';
import TagSummary from '@/components/TagSummary';
import CreateModal from '@/components/CreateModal';
import { useNotifier } from '@/components/Notifier';
import PageHeader from '@/components/PageHeader';
import { createModule, deleteModule, editModule, listModules } from '@/services/modules';
import EditModal from '@/components/EditModal';
import StatCard from '@/components/StatCard';
import ModuleCard from '@/components/modules/ModuleCard';

const ModulesList = () => {
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
  const [selectedRowKeys, setSelectedRowKeys] = useState<React.Key[]>([]);
  const [editModalOpen, setEditModalOpen] = useState(false);
  const [editingModule, setEditingModule] = useState<Module | null>(null);
  const [isAddModalOpen, setIsAddModalOpen] = useState(false);
  const [newModule, setNewModule] = useState<Partial<Module>>({
    code: '',
    year: new Date().getFullYear(),
    description: '',
    credits: 16,
  });
  const [viewMode, setViewMode] = useState<'table' | 'grid'>('grid');
  const [, setWindowWidth] = useState(window.innerWidth);

  // ======================================================================
  // ============================ Fetch Modules ===========================
  // ======================================================================

  const fetchModules = async () => {
    setLoading(true);
    const sort: SortOption[] = sorterState.map(({ field, order }) => ({ field, order }));

    const res = await listModules({
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
      notifyError('Failed to fetch modules', res.message);
    }

    setLoading(false);
  };

  useEffect(() => {
    const handleResize = () => {
      const width = window.innerWidth;
      setWindowWidth(width);

      if (width < 640) {
        setViewMode('grid');
      } else {
        const stored = localStorage.getItem('modules_view_mode');
        if (stored === 'table' || stored === 'grid') {
          setViewMode(stored);
        }
      }
    };

    window.addEventListener('resize', handleResize);
    handleResize(); // run once on mount

    return () => window.removeEventListener('resize', handleResize);
  }, []);

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

    const res = await createModule(payload);
    if (res.success) {
      notifySuccess('Module created successfully', res.message);
      setIsAddModalOpen(false);
      fetchModules();
    } else {
      notifyError('Failed to create module', res.message);
    }
  };

  const handleEditSave = async () => {
    if (!editingModule) return;

    const payload = {
      code: editingModule.code ?? '',
      year: Number(editingModule.year),
      description: editingModule.description ?? '',
      credits: Number(editingModule.credits) || 0,
    };

    const res = await editModule(editingModule.id, payload);
    if (res.success) {
      notifySuccess('Module updated successfully', res.message);
      setEditModalOpen(false);
      setEditingModule(null);
      fetchModules();
    } else {
      notifyError('Failed to update module', res.message);
    }
  };

  const handleDelete = async (id: number) => {
    const res = await deleteModule(id);
    if (res.success) {
      notifySuccess('Module deleted', res.message);
      fetchModules();
    } else {
      notifyError('Delete failed', res.message);
    }
  };

  const handleBulkDelete = async () => {
    for (const id of selectedRowKeys) {
      await handleDelete(Number(id));
    }
    setSelectedRowKeys([]);
    if (selectedRowKeys.length > 0) {
      notifySuccess(
        'Selected modules deleted',
        `${selectedRowKeys.length} module(s) have been deleted.`,
      );
    }
  };

  // ======================================================================
  // =========================== Table Columns ============================
  // ======================================================================

  const currentYear = new Date().getFullYear();
  const yearOptions = Array.from({ length: 10 }, (_, i) => String(currentYear - i));

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
        const { setSelectedKeys, selectedKeys, confirm } = props;
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
                  setSelectedKeys([]);
                  confirm({ closeDropdown: true });
                }}
              >
                Reset
              </Button>
            </div>
          </div>
        );
      },

      render: (_, record) => record.code.replace(/([A-Za-z]+)(\d+)/, '$1 $2'),
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
        const { setSelectedKeys, selectedKeys, confirm } = props;

        return (
          <div className="flex flex-col gap-2 p-2 w-56">
            <Select
              placeholder="Select year"
              value={selectedKeys[0]}
              onChange={(value) => setSelectedKeys([value])}
              style={{ width: '100%' }}
              options={yearOptions.map((y) => ({ label: y, value: y }))}
            />
            <div className="flex justify-between gap-2 mt-2">
              <Button type="primary" size="small" onClick={() => confirm()}>
                Filter
              </Button>
              <Button
                size="small"
                onClick={() => {
                  setSelectedKeys([]);
                  confirm({ closeDropdown: true });
                }}
              >
                Reset
              </Button>
            </div>
          </div>
        );
      },
      render: (_, record) => record.year,
    },
    {
      title: 'Description',
      dataIndex: 'description',
      key: 'description',
      sorter: { multiple: 3 },
      sortOrder: sorterState.find((s) => s.field === 'description')?.order ?? null,
      render: (_, record) => record.description,
    },
    {
      title: 'Credits',
      dataIndex: 'credits',
      key: 'credits',
      sorter: { multiple: 4 },
      sortOrder: sorterState.find((s) => s.field === 'credits')?.order ?? null,
      render: (_, record) => record.credits,
    },
    {
      title: 'Actions',
      key: 'actions',
      align: 'right',
      width: 100,
      render: (_, record) => (
        <div
          onClick={(e) => {
            e.stopPropagation(); // fully prevent row click on dropdown interaction
          }}
        >
          <Dropdown
            trigger={['click']}
            menu={{
              items: [
                {
                  key: 'view',
                  icon: <EyeOutlined />,
                  label: 'View',
                },
                {
                  key: 'edit',
                  icon: <EditOutlined />,
                  label: 'Edit',
                },
                {
                  key: 'delete',
                  icon: <DeleteOutlined />,
                  danger: true,
                  label: (
                    <Popconfirm
                      title="Delete this module?"
                      onConfirm={(e) => {
                        e?.stopPropagation();
                        handleDelete(record.id);
                      }}
                      onCancel={(e) => e?.stopPropagation()}
                      okText="Yes"
                      cancelText="No"
                    >
                      <span onClick={(e) => e.stopPropagation()}>Delete</span>
                    </Popconfirm>
                  ),
                },
              ],
              onClick: ({ key, domEvent }) => {
                domEvent.preventDefault();
                domEvent.stopPropagation();

                if (key === 'view') navigate(`/modules/${record.id}`);
                else if (key === 'edit') {
                  setEditingModule(record);
                  setEditModalOpen(true);
                }
              },
            }}
          >
            <Button
              icon={<MoreOutlined />}
              style={{ borderRadius: 6 }}
              onClick={(e) => e.stopPropagation()}
            />
          </Dropdown>
        </div>
      ),
    },
  ];

  // ======================================================================
  // ============================== Render ================================
  // ======================================================================

  return (
    <div className="bg-gray-50 dark:bg-gray-950 p-4 sm:p-6 h-full">
      <PageHeader title="Modules" description="All the modules in the COS department" />
      <Row gutter={[16, 16]} className="mb-6">
        <Col xs={24} sm={12} md={6}>
          <StatCard title="Total Modules" value={42} />
        </Col>
        <Col xs={24} sm={12} md={6}>
          <StatCard title="Modules This Year" value={12} />
        </Col>
        <Col xs={24} sm={12} md={6}>
          <StatCard title="Unique Years" value={6} />
        </Col>
        <Col xs={24} sm={12} md={6}>
          <StatCard title="Avg Credits" value={18} />
        </Col>
      </Row>

      <ControlBar
        handleSearch={setSearchTerm}
        searchTerm={searchTerm}
        viewMode={viewMode}
        onViewModeChange={(val) => {
          setViewMode(val);
          localStorage.setItem('modules_view_mode', val);
        }}
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
        {...(viewMode === 'grid' && {
          // Updated sort options
          sortOptions: [
            { label: 'Code', field: 'code' },
            { label: 'Year', field: 'year' },
            { label: 'Description', field: 'description' },
            { label: 'Credits', field: 'credits' },
          ],
          currentSort: sorterState.map((s) => `${s.field}.${s.order}`),
          onSortChange: (vals: string[]) => {
            setSorterState(
              vals.map((v) => {
                const [field, order] = v.split('.');
                return { field, order: order as 'ascend' | 'descend' };
              }),
            );
          },

          // Updated filter groups
          filterGroups: [
            {
              key: 'year',
              label: 'Year',
              type: 'select',
              options: yearOptions.map((y) => ({ label: y, value: y })),
            },
            {
              key: 'code',
              label: 'Code',
              type: 'text',
            },
          ],
          activeFilters: Object.entries(filterState).flatMap(([key, vals]) =>
            vals.map((v) => `${key}:${v}`),
          ),
          onFilterChange: (vals: string[]) => {
            const updated: Record<string, string[]> = {};
            vals.forEach((v) => {
              const [key, value] = v.split(':');
              if (!updated[key]) updated[key] = [];
              updated[key].push(value);
            });
            setFilterState(updated);
          },
        })}
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

      {viewMode === 'table' ? (
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
          size="middle"
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
          onRow={(record) => ({
            onClick: () => {
              navigate(`/modules/${record.id}`);
            },
          })}
          rowClassName={() => 'cursor-pointer hover:bg-gray-50 dark:hover:bg-gray-800'}
          className="border-1 bg-white dark:bg-gray-950 border-gray-200 dark:border-gray-800 rounded-lg overflow-hidden"
        />
      ) : modules.length === 0 ? (
        <Empty
          image={Empty.PRESENTED_IMAGE_SIMPLE}
          description="No modules found."
          className="mt-10"
        >
          <Button icon={<ReloadOutlined />} onClick={clearAll}>
            Clear All Filters
          </Button>
        </Empty>
      ) : (
        <div className="grid grid-cols-1 sm:grid-cols-2 md:grid-cols-3 lg:grid-cols-4 gap-4">
          {modules.map((mod) => (
            <ModuleCard
              key={mod.id}
              module={{ ...mod, role: 'Student' }}
              onToggleFavorite={() => {}}
              isFavorite={false}
              actions={[
                <Tooltip title="View" key="view">
                  <EyeOutlined
                    onClick={(e) => {
                      e.stopPropagation();
                      navigate(`/modules/${mod.id}`);
                    }}
                  />
                </Tooltip>,
                <Tooltip title="Edit" key="edit">
                  <EditOutlined
                    onClick={(e) => {
                      e.stopPropagation();
                      setEditingModule(mod);
                      setEditModalOpen(true);
                    }}
                  />
                </Tooltip>,
                <Tooltip title="Delete" key="delete">
                  <Popconfirm
                    title="Delete this module?"
                    onConfirm={(e) => {
                      e?.stopPropagation();
                      handleDelete(mod.id);
                    }}
                    onCancel={(e) => e?.stopPropagation()}
                    okText="Yes"
                    cancelText="No"
                  >
                    <DeleteOutlined onClick={(e) => e.stopPropagation()} />
                  </Popconfirm>
                </Tooltip>,
              ]}
              showFavorite={false}
            />
          ))}
        </div>
      )}

      <CreateModal
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

      {editingModule && (
        <EditModal
          open={editModalOpen}
          onCancel={() => setEditModalOpen(false)}
          onEdit={handleEditSave}
          initialValues={editingModule}
          onChange={(val) => setEditingModule({ ...editingModule, ...val })}
          title="Edit Module"
          fields={[
            { name: 'code', label: 'Module Code', type: 'text', required: true },
            { name: 'year', label: 'Year', type: 'number', required: true },
            { name: 'description', label: 'Description', type: 'text', required: true },
            { name: 'credits', label: 'Credits', type: 'number', required: true },
          ]}
        />
      )}
    </div>
  );
};

export default ModulesList;
