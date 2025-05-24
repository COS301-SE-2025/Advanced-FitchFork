import {
  Table,
  Input,
  Button,
  message,
  Space,
  Card,
  Statistic,
  Dropdown,
  Popconfirm,
  Empty,
} from 'antd';
import type { MenuProps, TableColumnsType, TablePaginationConfig, TableProps } from 'antd';
import { ReloadOutlined, EditOutlined, DeleteOutlined } from '@ant-design/icons';
import { useState } from 'react';
import type { FilterValue } from 'antd/es/table/interface';
import TableTagSummary from '@/components/TableTagSummary';
import AppLayout from '@/layouts/AppLayout';
import { mockModules } from '@/mocks/modules';
import type { Module } from '@/types/modules';
import { useNavigate } from 'react-router-dom';

const { Search } = Input;

const AdminModuleTableView: React.FC = () => {
  const navigate = useNavigate();
  const [modules, setModules] = useState<Module[]>(mockModules);
  const [filteredModules, setFilteredModules] = useState<Module[]>(mockModules);
  const [editingRowId, setEditingRowId] = useState<number | null>(null);
  const [editCache, setEditCache] = useState<Partial<Module>>({});
  const [selectedRowKeys, setSelectedRowKeys] = useState<React.Key[]>([]);
  const [searchTerm, setSearchTerm] = useState('');
  const [filterState, setFilterState] = useState<Record<string, FilterValue | null>>({});
  const [sorterState, setSorterState] = useState<any[]>([]);
  const [pagination, setPagination] = useState<TablePaginationConfig>({
    current: 1,
    pageSize: 5,
    pageSizeOptions: ['5', '10', '20', '50', '100'],
    showSizeChanger: true,
    showQuickJumper: true,
    showTotal: (total, range) => `${range[0]}-${range[1]} of ${total} modules`,
    style: {
      paddingRight: '20px',
    },
  });

  const handleSearch = (value: string) => {
    setSearchTerm(value);
    const lower = value.toLowerCase();
    const filtered = modules.filter(
      (m) => m.code.toLowerCase().includes(lower) || m.description.toLowerCase().includes(lower),
    );
    setFilteredModules(filtered);
    setPagination((prev) => ({ ...prev, current: 1 }));
  };

  const handleEdit = (record: Module) => {
    setEditingRowId(record.id);
    setEditCache({ ...record });
  };

  const handleEditSave = () => {
    if (!editCache.code || !editCache.year || !editCache.description) {
      message.error('All fields must be filled');
      return;
    }
    const updated = modules.map((m) => (m.id === editingRowId ? { ...m, ...editCache } : m));
    setModules(updated);
    setFilteredModules(updated);
    setEditingRowId(null);
    setEditCache({});
    message.success('Module updated');
  };

  const handleDelete = (id: number) => {
    const updated = modules.filter((m) => m.id !== id);
    setModules(updated);
    setFilteredModules(updated);
    message.success('Module deleted');
  };

  const handleAddModule = () => {
    const now = new Date();
    const newModule: Module = {
      id: Date.now(),
      code: '',
      year: new Date().getFullYear(),
      description: '',
      credits: 0,
      created_at: now,
      updated_at: now,
    };
    setModules([newModule, ...modules]);
    setFilteredModules([newModule, ...filteredModules]);
    setEditingRowId(newModule.id);
    setEditCache({ ...newModule });
  };

  const handleBulkDelete = () => {
    const updated = modules.filter((m) => !selectedRowKeys.includes(m.id));
    setModules(updated);
    setFilteredModules(updated);
    setSelectedRowKeys([]);
    message.success('Selected modules deleted');
  };

  const clearSearch = () => {
    setSearchTerm('');
    setFilteredModules(modules);
  };

  const clearAllFilters = () => {
    clearSearch();
    setFilterState({});
    setSorterState([]);
  };

  const columns: TableColumnsType<Module> = [
    {
      title: 'Code',
      dataIndex: 'code',
      key: 'code',
      sorter: {
        compare: (a, b) => a.code.localeCompare(b.code),
        multiple: 1,
      },
      filteredValue: filterState?.code || null,
      sortOrder: sorterState.find((s) => s.columnKey === 'code')?.order || null,
      onFilter: (value, record) =>
        typeof value === 'string' &&
        record.code.toLowerCase().includes(value.toLowerCase().replace(/\s+/g, '')),
      filterDropdown: ({ setSelectedKeys, selectedKeys, confirm, clearFilters }) => (
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
                confirm({ closeDropdown: true }); // ensures filter is cleared AND dropdown closes
              }}
            >
              Reset
            </Button>
          </div>
        </div>
      ),
      render: (_, record) =>
        editingRowId === record.id ? (
          <Input
            value={editCache.code}
            onChange={(e) => setEditCache({ ...editCache, code: e.target.value })}
          />
        ) : (
          // Insert a space between letters and numbers (e.g., COS332 -> COS 332)
          record.code.replace(/([A-Za-z]+)(\d+)/, '$1 $2')
        ),
    },
    {
      title: 'Year',
      dataIndex: 'year',
      key: 'year',
      sorter: {
        compare: (a, b) => a.year - b.year,
        multiple: 2,
      },
      sortOrder: sorterState.find((s) => s.columnKey === 'year')?.order || null,
      filteredValue: filterState?.year || null,
      onFilter: (value, record) => record.year === Number(value),
      filterDropdown: ({ setSelectedKeys, selectedKeys, confirm, clearFilters }) => (
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
                confirm({ closeDropdown: true }); // ensures filter is cleared AND dropdown closes
              }}
            >
              Reset
            </Button>
          </div>
        </div>
      ),
      render: (_, record) =>
        editingRowId === record.id ? (
          <Input
            type="number"
            value={editCache.year}
            onChange={(e) => setEditCache({ ...editCache, year: parseInt(e.target.value) })}
          />
        ) : (
          record.year
        ),
    },
    {
      title: 'Description',
      dataIndex: 'description',
      key: 'description',
      sorter: {
        compare: (a, b) => a.description.localeCompare(b.description),
        multiple: 3,
      },
      sortOrder: sorterState.find((s) => s.columnKey === 'description')?.order || null,
      render: (_, record) =>
        editingRowId === record.id ? (
          <Input
            value={editCache.description}
            onChange={(e) => setEditCache({ ...editCache, description: e.target.value })}
          />
        ) : (
          record.description
        ),
    },
    {
      title: 'Actions',
      key: 'actions',
      render: (_, record) =>
        editingRowId === record.id ? (
          <Space>
            <Button type="primary" size="small" onClick={handleEditSave}>
              Save
            </Button>
            <Button size="small" onClick={() => setEditingRowId(null)}>
              Cancel
            </Button>
          </Space>
        ) : (
          <Space>
            <Button icon={<EditOutlined />} size="small" onClick={() => handleEdit(record)} />
            <Popconfirm
              title="Delete this module?"
              onConfirm={() => handleDelete(record.id)}
              okText="Yes"
              cancelText="No"
            >
              <Button icon={<DeleteOutlined />} danger size="small" />
            </Popconfirm>
          </Space>
        ),
    },
  ];

  const clearMenuItems: MenuProps['items'] = [
    {
      key: 'clear-search',
      label: 'Clear Search',
      onClick: clearSearch,
    },
    {
      key: 'clear-filters',
      label: 'Clear Filters',
      onClick: () => {
        setFilterState({});
      },
    },
    {
      key: 'clear-sorts',
      label: 'Clear Sorts',
      onClick: () => {
        setSorterState([]);
      },
    },
    {
      key: 'clear-all',
      label: 'Clear All',
      onClick: () => {
        clearAllFilters();
      },
    },
  ];

  const rowSelection: TableProps<Module>['rowSelection'] = {
    selectedRowKeys,
    onChange: (keys) => setSelectedRowKeys(keys),
  };

  return (
    <AppLayout title="Modules" description="All the modules in the COS department">
      {/* Stats */}
      <div className="mb-6 flex flex-wrap gap-4">
        <Card className="flex-1 min-w-[200px]">
          <Statistic title="Total Modules" value={modules.length} />
        </Card>
      </div>

      {/* Control Bar */}
      <div className="mb-4 flex flex-wrap items-center justify-between gap-4">
        <Search
          placeholder="Search by code or description"
          allowClear
          onChange={(e) => handleSearch(e.target.value)}
          value={searchTerm}
          style={{ maxWidth: 320 }}
          className="w-full sm:w-auto"
        />

        <div className="flex flex-wrap gap-2 items-center">
          <Button type="primary" onClick={handleAddModule}>
            Add Module
          </Button>
          <Dropdown menu={{ items: clearMenuItems }}>
            <Button icon={<ReloadOutlined />}>Clear</Button>
          </Dropdown>
          {selectedRowKeys.length > 0 && (
            <Popconfirm
              title="Delete selected modules?"
              onConfirm={handleBulkDelete}
              okText="Yes"
              cancelText="No"
            >
              <Button danger icon={<DeleteOutlined />}>
                Delete Selected
              </Button>
            </Popconfirm>
          )}
        </div>
      </div>

      {/* Active Filters / Sorts / Search Summary Tags */}
      <TableTagSummary
        searchTerm={searchTerm}
        onClearSearch={clearSearch}
        filters={filterState}
        onClearFilter={(key) =>
          setFilterState((prev) => {
            const updated = { ...prev };
            delete updated[key];
            return updated;
          })
        }
        sorters={sorterState}
        onClearSorter={(key) => setSorterState((prev) => prev.filter((s) => s.columnKey !== key))}
      />

      {/* Table */}
      <Table<Module>
        rowKey="id"
        columns={columns}
        dataSource={filteredModules}
        rowSelection={rowSelection}
        pagination={pagination}
        onRow={(record) => ({
          onClick: () => navigate(`/modules/${record.id}`),
          style: { cursor: 'pointer' },
        })}
        onChange={(p, filters, sorter) => {
          setPagination(p);
          setFilterState(filters);
          const sorterArray = Array.isArray(sorter) ? sorter : [sorter];
          setSorterState(sorterArray);
        }}
        locale={{
          emptyText: (
            <Empty image={Empty.PRESENTED_IMAGE_SIMPLE} description="No modules found.">
              <Button
                icon={<ReloadOutlined />}
                onClick={() => {
                  clearAllFilters();
                }}
              >
                Clear All Filters
              </Button>
            </Empty>
          ),
        }}
        className="border-1 border-gray-100 dark:border-gray-800 rounded-lg overflow-hidden"
      />
    </AppLayout>
  );
};

export default AdminModuleTableView;
