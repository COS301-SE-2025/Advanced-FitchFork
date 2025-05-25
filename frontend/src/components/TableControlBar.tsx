import { Input, Button, Dropdown, Popconfirm, type MenuProps } from 'antd';
import { ReloadOutlined, DeleteOutlined, PlusOutlined } from '@ant-design/icons'; // <-- Add this
import React from 'react';

const { Search } = Input;

interface Props {
  handleSearch: (key: string) => void;
  searchTerm: string;
  handleAdd?: () => void;
  handleBulkDelete?: () => void;
  clearMenuItems?: MenuProps['items'];
  selectedRowKeys?: React.Key[];
  searchPlaceholder?: string;
  addButtonText?: string;
  addButtonVisible?: boolean;
  bulkDeleteVisible?: boolean;
  bulkDeleteConfirmMessage?: string;
}

const TableControlBar: React.FC<Props> = ({
  handleSearch,
  searchTerm,
  handleAdd,
  handleBulkDelete,
  clearMenuItems = [],
  selectedRowKeys = [],
  searchPlaceholder = 'Search...',
  addButtonText = 'Add',
  addButtonVisible = true,
  bulkDeleteVisible = true,
  bulkDeleteConfirmMessage = 'Delete selected items?',
}) => {
  return (
    <div className="mb-4 flex flex-wrap items-center justify-between gap-4">
      <Search
        placeholder={searchPlaceholder}
        allowClear
        onChange={(e) => handleSearch(e.target.value)}
        value={searchTerm}
        style={{ maxWidth: 320 }}
        className="w-full sm:w-auto"
      />

      <div className="flex flex-wrap gap-2 items-center">
        {addButtonVisible && handleAdd && (
          <Button type="primary" onClick={handleAdd} icon={<PlusOutlined />}>
            {addButtonText}
          </Button>
        )}

        {clearMenuItems.length > 0 && (
          <Dropdown menu={{ items: clearMenuItems }}>
            <Button icon={<ReloadOutlined />}>Clear</Button>
          </Dropdown>
        )}

        {bulkDeleteVisible && selectedRowKeys.length > 0 && handleBulkDelete && (
          <Popconfirm
            title={bulkDeleteConfirmMessage}
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
  );
};

export default TableControlBar;
