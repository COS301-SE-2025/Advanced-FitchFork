import React, { useState } from 'react';
import {
  Input,
  Button,
  Dropdown,
  Popconfirm,
  Segmented,
  Select,
  type MenuProps,
  Modal,
  Col,
  Row,
} from 'antd';
import {
  ReloadOutlined,
  DeleteOutlined,
  PlusOutlined,
  TableOutlined,
  AppstoreOutlined,
  FilterOutlined,
  SortAscendingOutlined,
  ThunderboltOutlined,
} from '@ant-design/icons';

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
  options?: { label: string; value: string }[]; // for select and multi-select
}

interface Props {
  handleSearch: (key: string) => void;
  searchTerm: string;
  viewMode?: 'table' | 'grid';
  onViewModeChange?: (val: 'table' | 'grid') => void;
  handleAdd?: () => void;
  handleBulkDelete?: () => void;
  clearMenuItems?: MenuProps['items'];
  selectedRowKeys?: React.Key[];
  searchPlaceholder?: string;
  addButtonText?: string;
  addButtonVisible?: boolean;
  bulkDeleteVisible?: boolean;
  bulkDeleteConfirmMessage?: string;

  sortOptions?: SortOption[];
  currentSort?: string[];
  onSortChange?: (value: string[]) => void;

  filterGroups?: FilterGroup[];
  activeFilters?: string[];
  onFilterChange?: (values: string[]) => void;

  actions?: React.ReactNode;
}

const ControlBar: React.FC<Props> = ({
  handleSearch,
  searchTerm,
  viewMode,
  onViewModeChange,
  handleAdd,
  handleBulkDelete,
  clearMenuItems = [],
  selectedRowKeys = [],
  searchPlaceholder = 'Search...',
  addButtonText = 'Add',
  addButtonVisible = true,
  bulkDeleteVisible = true,
  bulkDeleteConfirmMessage = 'Delete selected items?',
  sortOptions = [],
  currentSort = [],
  onSortChange,
  filterGroups = [],
  activeFilters = [],
  onFilterChange,
  actions,
}) => {
  const [sortModalOpen, setSortModalOpen] = useState(false);
  const [filterModalOpen, setFilterModalOpen] = useState(false);

  return (
    <div className="bg-white dark:bg-gray-950 p-2 rounded-lg border border-gray-200 dark:border-gray-800 mb-4 flex flex-col sm:flex-row sm:items-center sm:justify-between gap-4">
      {/* Left: View mode + search (shared) */}
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

      {/* Mobile-only Add button row */}
      {addButtonVisible && handleAdd && (
        <div className="sm:hidden">
          <Button type="primary" onClick={handleAdd} icon={<PlusOutlined />} className="w-full">
            {addButtonText}
          </Button>
        </div>
      )}
      <Row gutter={8} align="middle" wrap={false}>
        {actions && <Col>{actions}</Col>}

        {(sortOptions.length > 0 || filterGroups.length > 0) && (
          <Col flex="1">
            <Dropdown
              menu={{
                items: [
                  ...(sortOptions.length > 0
                    ? [
                        {
                          key: 'sort',
                          label: 'Sort',
                          icon: <SortAscendingOutlined />,
                          onClick: () => setSortModalOpen(true),
                        },
                      ]
                    : []),
                  ...(filterGroups.length > 0
                    ? [
                        {
                          key: 'filters',
                          label: 'Filters',
                          icon: <FilterOutlined />,
                          onClick: () => setFilterModalOpen(true),
                        },
                      ]
                    : []),
                ],
              }}
            >
              <Button block icon={<ThunderboltOutlined />}>
                Actions
              </Button>
            </Dropdown>
          </Col>
        )}

        {/* Desktop placement */}
        {addButtonVisible && handleAdd && (
          <div className="hidden sm:block">
            <Button
              type="primary"
              onClick={handleAdd}
              icon={<PlusOutlined />}
              className="whitespace-nowrap"
            >
              {addButtonText}
            </Button>
          </div>
        )}
        {clearMenuItems.length > 0 && (
          <Col flex="0 0 120px">
            <Dropdown menu={{ items: clearMenuItems }}>
              <Button block icon={<ReloadOutlined />}>
                Clear
              </Button>
            </Dropdown>
          </Col>
        )}

        {bulkDeleteVisible && selectedRowKeys.length > 0 && handleBulkDelete && (
          <Col flex="0 0 160px">
            <Popconfirm
              title={bulkDeleteConfirmMessage || 'Delete selected items?'}
              onConfirm={handleBulkDelete}
              okText="Yes"
              cancelText="No"
            >
              <Button block danger icon={<DeleteOutlined />}>
                Delete Selected
              </Button>
            </Popconfirm>
          </Col>
        )}
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
