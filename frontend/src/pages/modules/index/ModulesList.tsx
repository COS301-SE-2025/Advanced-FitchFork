import { useNavigate } from 'react-router-dom';
import type { Module } from '@/types/modules';
import { listModules, createModule, editModule, deleteModule } from '@/services/modules';
import { useNotifier } from '@/components/Notifier';
import PageHeader from '@/components/PageHeader';
import StatCard from '@/components/StatCard';
import ModuleCard from '@/components/modules/ModuleCard';
import type { SortOption } from '@/types/common';
import { EntityList } from '@/components/EntityList';
import ModuleCreditsTag from '@/components/modules/ModuleCreditsTag';

const currentYear = new Date().getFullYear();
const yearOptions = Array.from({ length: 10 }, (_, i) => String(currentYear - i));

const ModulesList = () => {
  const navigate = useNavigate();
  const { notifySuccess, notifyError } = useNotifier();

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
    sort: SortOption[];
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
      notifyError('Failed to fetch modules', res.message);
      return { items: [], total: 0 };
    }
  };

  const handleCreate = async (values: Partial<Module>): Promise<void> => {
    const res = await createModule({
      code: values.code ?? '',
      year: Number(values.year),
      description: values.description ?? '',
      credits: Number(values.credits),
    });

    if (res.success) {
      notifySuccess('Module created', res.message);
    } else {
      notifyError('Failed to create module', res.message);
    }
  };

  const handleEdit = async (item: Module, values: Partial<Module>): Promise<void> => {
    const res = await editModule(item.id, {
      code: values.code ?? '',
      year: Number(values.year),
      description: values.description ?? '',
      credits: Number(values.credits),
    });

    if (res.success) {
      notifySuccess('Module updated', res.message);
    } else {
      notifyError('Failed to update module', res.message);
    }
  };

  const handleDelete = async (mod: Module): Promise<void> => {
    const res = await deleteModule(mod.id);
    if (res.success) {
      notifySuccess('Module deleted', res.message);
    } else {
      notifyError('Delete failed', res.message);
    }
  };

  return (
    <div className="p-4 sm:p-6 h-full">
      <PageHeader title="Modules" description="All the modules in the COS department" />

      <div className="mb-6 grid grid-cols-1 sm:grid-cols-2 md:grid-cols-4 gap-4">
        <StatCard title="Total Modules" value={42} />
        <StatCard title="Modules This Year" value={12} />
        <StatCard title="Unique Years" value={6} />
        <StatCard title="Avg Credits" value={18} />
      </div>

      <EntityList<Module>
        name="Modules"
        fetchItems={fetchModules}
        getRowKey={(mod) => mod.id}
        onRowClick={(mod) => navigate(`/modules/${mod.id}`)}
        renderGridItem={(mod, actions) => (
          <ModuleCard
            key={mod.id}
            module={{ ...mod, role: 'Student' }}
            isFavorite={false}
            onToggleFavorite={() => {}}
            showFavorite={false}
            actions={actions}
          />
        )}
        onDelete={handleDelete}
        createModal={{
          title: 'Add Module',
          onCreate: handleCreate,
          getInitialValues: () => ({
            code: '',
            year: currentYear,
            description: '',
            credits: 16,
          }),
          fields: [
            { name: 'code', label: 'Module Code', type: 'text', required: true },
            { name: 'year', label: 'Year', type: 'number', required: true },
            { name: 'description', label: 'Description', type: 'text', required: true },
            { name: 'credits', label: 'Credits', type: 'number', required: true },
          ],
        }}
        editModal={{
          title: 'Edit Module',
          onEdit: handleEdit,
          fields: [
            { name: 'code', label: 'Module Code', type: 'text', required: true },
            { name: 'year', label: 'Year', type: 'number', required: true },
            { name: 'description', label: 'Description', type: 'text', required: true },
            { name: 'credits', label: 'Credits', type: 'number', required: true },
          ],
        }}
        columns={[
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
          },
          {
            title: 'Description',
            dataIndex: 'description',
            key: 'description',
            sorter: { multiple: 3 },
          },
          {
            title: 'Credits',
            dataIndex: 'credits',
            key: 'credits',
            sorter: { multiple: 4 },
            render: (_, record) => <ModuleCreditsTag credits={record.credits} />,
          },
        ]}
        sortOptions={[
          { label: 'Code', field: 'code' },
          { label: 'Year', field: 'year' },
          { label: 'Description', field: 'description' },
          { label: 'Credits', field: 'credits' },
        ]}
        filterGroups={[
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
        ]}
      />
    </div>
  );
};

export default ModulesList;
