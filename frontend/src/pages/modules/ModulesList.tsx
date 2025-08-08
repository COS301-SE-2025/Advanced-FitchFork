import { useState, useRef } from 'react';
import { useNavigate } from 'react-router-dom';
import type { Module } from '@/types/modules';
import { listModules, createModule, editModule, deleteModule } from '@/services/modules';
import { DeleteOutlined, EditOutlined, PlusOutlined } from '@ant-design/icons';
import PageHeader from '@/components/PageHeader';
import StatCard from '@/components/StatCard';
import ModuleCard from '@/components/modules/ModuleCard';
import ModuleCreditsTag from '@/components/modules/ModuleCreditsTag';
import { EntityList, type EntityListHandle, type EntityListProps } from '@/components/EntityList';
import CreateModal from '@/components/common/CreateModal';
import EditModal from '@/components/common/EditModal';
import { message } from '@/utils/message';
import dayjs from 'dayjs';
import { useAuth } from '@/context/AuthContext';

const currentYear = new Date().getFullYear();

const ModulesList = () => {
  const auth = useAuth();
  const navigate = useNavigate();
  const listRef = useRef<EntityListHandle>(null);

  const [createOpen, setCreateOpen] = useState(false);
  const [editOpen, setEditOpen] = useState(false);
  const [editingItem, setEditingItem] = useState<Module | null>(null);

  const fetchModules = async ({
    page,
    per_page,
    query,
    filters,
    sort,
  }: {
    page: number;
    per_page: number;
    query?: string;
    filters: Record<string, string[]>;
    sort: { field: string; order: 'ascend' | 'descend' }[];
  }): Promise<{ items: Module[]; total: number }> => {
    const res = await listModules({
      page,
      per_page,
      query,
      code: filters.code?.[0],
      year: filters.year?.[0] ? parseInt(filters.year[0]) : undefined,
      sort,
    });

    if (res.success) {
      return {
        items: res.data.modules,
        total: res.data.total,
      };
    } else {
      message.error(`Failed to fetch modules: ${res.message}`);
      return { items: [], total: 0 };
    }
  };

  const handleCreate = async (values: Record<string, any>) => {
    const res = await createModule({
      code: values.code,
      year: Number(values.year),
      description: values.description,
      credits: Number(values.credits),
    });

    if (res.success) {
      message.success(res.message || 'Module created');
      setCreateOpen(false);
      listRef.current?.refresh();
    } else {
      message.error(res.message);
    }
  };

  const handleEdit = async (values: Record<string, any>) => {
    if (!editingItem) return;

    const res = await editModule(editingItem.id, {
      code: values.code,
      year: Number(values.year),
      description: values.description,
      credits: Number(values.credits),
    });

    if (res.success) {
      message.success(res.message || 'Module updated');
      setEditOpen(false);
      setEditingItem(null);
      listRef.current?.refresh();
    } else {
      message.error(res.message);
    }
  };

  const handleDelete = async (mod: Module, refresh: () => void): Promise<void> => {
    const res = await deleteModule(mod.id);
    if (res.success) {
      message.success(res.message || 'Module deleted');
      refresh();
    } else {
      message.error(`Delete failed: ${res.message}`);
    }
  };

  const actions: EntityListProps<Module>['actions'] = auth.isAdmin
    ? {
        control: [
          {
            key: 'create',
            label: 'Add Module',
            icon: <PlusOutlined />,
            isPrimary: true,
            handler: ({ refresh }) => {
              setCreateOpen(true);
              refresh();
            },
          },
        ],
        entity: (entity: Module) => [
          {
            key: 'edit',
            label: 'Edit',
            icon: <EditOutlined />,
            handler: ({ refresh }) => {
              setEditingItem(entity);
              setEditOpen(true);
              refresh();
            },
          },
          {
            key: 'delete',
            label: 'Delete',
            icon: <DeleteOutlined />,
            confirm: true,
            handler: ({ refresh }) => {
              handleDelete(entity, refresh);
            },
          },
        ],
        bulk: [
          {
            key: 'bulk-delete',
            label: 'Bulk Delete',
            icon: <DeleteOutlined />,
            isPrimary: true,
            handler: ({ selected, refresh }) => {
              if (!selected || selected.length === 0) {
                message.warning('No modules selected');
                return;
              }
              message.info(`Bulk delete not implemented yet. ${selected.length} items selected.`);
              refresh();
            },
          },
          {
            key: 'bulk-edit',
            label: 'Bulk Edit',
            icon: <EditOutlined />,
            handler: ({ selected, refresh }) => {
              if (!selected || selected.length === 0) {
                message.warning('No modules selected');
                return;
              }
              message.info(`Bulk edit not implemented yet. ${selected.length} items selected.`);
              refresh();
            },
          },
        ],
      }
    : undefined;

  return (
    <div className="h-full flex flex-col overflow-hidden">
      <div className="flex-1 overflow-y-auto p-4 sm:p-6">
        <PageHeader title="Modules" description="All the modules in the COS department" />

        <div className="mb-6 grid grid-cols-1 sm:grid-cols-2 md:grid-cols-4 gap-4">
          <StatCard title="Total Modules" value={42} />
          <StatCard title="Modules This Year" value={12} />
          <StatCard title="Unique Years" value={6} />
          <StatCard title="Avg Credits" value={18} />
        </div>

        <EntityList<Module>
          ref={listRef}
          name="Modules"
          defaultViewMode={auth.isUser ? 'grid' : 'table'}
          fetchItems={fetchModules}
          getRowKey={(mod) => mod.id}
          onRowClick={(mod) => navigate(`/modules/${mod.id}`)}
          renderGridItem={(mod, actions) => (
            <ModuleCard
              key={mod.id}
              module={mod}
              isFavorite={false}
              onToggleFavorite={() => {}}
              showFavorite={false}
              actions={actions}
            />
          )}
          // listMode
          // renderListItem={(mod) => (
          //   <List.Item
          //     key={mod.id}
          //     onClick={() => navigate(`/modules/${mod.id}`)}
          //     className="cursor-pointer hover:bg-gray-100 dark:hover:bg-gray-800 transition-colors"
          //   >
          //     <List.Item.Meta
          //       title={
          //         <div className="flex justify-between items-center">
          //           <span className="font-semibold">
          //             {mod.code.replace(/([A-Za-z]+)(\d+)/, '$1 $2')} ({mod.year})
          //           </span>
          //           <ModuleCreditsTag credits={mod.credits} />
          //         </div>
          //       }
          //       description={mod.description}
          //     />
          //   </List.Item>
          // )}
          columnToggleEnabled
          actions={actions}
          columns={[
            {
              title: 'ID',
              dataIndex: 'id',
              key: 'id',
              sorter: { multiple: 0 },
              defaultHidden: true,
            },
            {
              title: 'Code',
              dataIndex: 'code',
              key: 'code',
              sorter: { multiple: 1 },
              render: (_: unknown, record: Module) =>
                record.code.replace(/([A-Za-z]+)(\d+)/, '$1 $2'),
            },
            {
              title: 'Year',
              dataIndex: 'year',
              key: 'year',
              sorter: { multiple: 2 },
              filters: Array.from({ length: 10 }, (_, i) => {
                const year = String(currentYear - i);
                return { text: year, value: year };
              }),
            },
            {
              title: 'Description',
              dataIndex: 'description',
              key: 'description',
              sorter: { multiple: 3 },
              defaultHidden: true,
            },
            {
              title: 'Credits',
              dataIndex: 'credits',
              key: 'credits',
              sorter: { multiple: 4 },
              render: (_, record) => <ModuleCreditsTag credits={record.credits} />,
            },
            {
              title: 'Created At',
              dataIndex: 'created_at',
              key: 'created_at',
              sorter: { multiple: 5 },
              defaultHidden: true,
              render: (value: string) => dayjs(value).format('YYYY-MM-DD HH:mm'),
            },
            {
              title: 'Updated At',
              dataIndex: 'updated_at',
              key: 'updated_at',
              sorter: { multiple: 6 },
              defaultHidden: true,
              render: (value: string) => dayjs(value).format('YYYY-MM-DD HH:mm'),
            },
          ]}
        />

        <CreateModal
          open={createOpen}
          onCancel={() => setCreateOpen(false)}
          onCreate={handleCreate}
          initialValues={{
            code: '',
            year: currentYear,
            description: '',
            credits: 16,
          }}
          fields={[
            { name: 'code', label: 'Module Code', type: 'text', required: true },
            { name: 'year', label: 'Year', type: 'number', required: true },
            { name: 'description', label: 'Description', type: 'text', required: true },
            { name: 'credits', label: 'Credits', type: 'number', required: true },
          ]}
          title="Add Module"
        />

        <EditModal
          open={editOpen}
          onCancel={() => {
            setEditOpen(false);
            setEditingItem(null);
          }}
          onEdit={handleEdit}
          initialValues={{
            code: editingItem?.code ?? '',
            year: editingItem?.year ?? currentYear,
            description: editingItem?.description ?? '',
            credits: editingItem?.credits ?? 16,
          }}
          fields={[
            { name: 'code', label: 'Module Code', type: 'text', required: true },
            { name: 'year', label: 'Year', type: 'number', required: true },
            { name: 'description', label: 'Description', type: 'text', required: true },
            { name: 'credits', label: 'Credits', type: 'number', required: true },
          ]}
          title="Edit Module"
        />
      </div>
    </div>
  );
};

export default ModulesList;
