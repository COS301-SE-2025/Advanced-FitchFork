import { useState } from 'react';
import dayjs from 'dayjs';
import { useNavigate } from 'react-router-dom';
import { Button } from 'antd';

import PageHeader from '@/components/PageHeader';
import { useNotifier } from '@/components/Notifier';
import { useModule } from '@/context/ModuleContext';
import { EntityList } from '@/components/EntityList';
import AssignmentTypeTag from '@/components/assignments/AssignmentTypeTag';
import AssignmentStatusTag from '@/components/assignments/AssignmentStatusTag';
import AssignmentCard from '@/components/assignments/AssignmentCard';
import AssignmentSetup from '@/pages/modules/assignments/steps/AssignmentSetup';

import { listAssignments, editAssignment, deleteAssignment } from '@/services/modules/assignments';

import {
  type Assignment,
  type AssignmentType,
  ASSIGNMENT_TYPES,
} from '@/types/modules/assignments';
import { getRandomAssignmentStatus } from '@/constants/mock/assignment';
import type { SortOption } from '@/types/common';

const AssignmentsList = () => {
  const module = useModule();
  const navigate = useNavigate();
  const { notifySuccess, notifyError } = useNotifier();

  const [setupOpen, setSetupOpen] = useState(false);

  const fetchAssignments = async ({
    page,
    per_page,
    query,
    sort,
    filters,
  }: {
    page: number;
    per_page: number;
    query?: string;
    filters: Record<string, string[]>;
    sort: SortOption[];
  }): Promise<{ items: Assignment[]; total: number }> => {
    const res = await listAssignments(module.id, {
      page,
      per_page,
      query,
      sort,
      name: filters.name?.[0],
      assignment_type: filters.assignment_type?.[0] as AssignmentType | undefined,
    });

    if (res.success) {
      return {
        items: res.data.assignments.map((a) => ({
          ...a,
          status: getRandomAssignmentStatus(),
        })),
        total: res.data.total,
      };
    } else {
      notifyError('Failed to fetch assignments', res.message);
      return { items: [], total: 0 };
    }
  };

  const handleEdit = async (item: Assignment, values: Record<string, any>) => {
    const res = await editAssignment(module.id, item.id, {
      name: values.name,
      assignment_type: values.assignment_type,
      available_from: dayjs(values.available_from).toISOString(),
      due_date: dayjs(values.due_date).toISOString(),
      description: values.description,
    });

    if (res.success) {
      notifySuccess('Updated', 'Assignment updated successfully');
    } else {
      notifyError('Update failed', res.message);
    }
  };

  const handleDelete = async (assignment: Assignment) => {
    const res = await deleteAssignment(module.id, assignment.id);

    if (res.success) {
      notifySuccess('Deleted', 'Assignment removed successfully');
    } else {
      notifyError('Delete failed', res.message);
    }
  };

  return (
    <div className="p-4 sm:p-6">
      <PageHeader title="Assignments" description={`All the assignments for ${module.code}`} />

      <EntityList<Assignment>
        name="Assignments"
        fetchItems={fetchAssignments}
        getRowKey={(a) => a.id}
        onRowClick={(a) => navigate(`/modules/${module.id}/assignments/${a.id}/submissions`)}
        onDelete={handleDelete}
        renderGridItem={(assignment, actions) => (
          <AssignmentCard key={assignment.id} assignment={assignment} actions={actions} />
        )}
        editModal={{
          title: 'Edit Assignment',
          onEdit: handleEdit,
          fields: [
            { name: 'name', label: 'Assignment Name', type: 'text', required: true },
            {
              name: 'assignment_type',
              label: 'Type',
              type: 'select',
              required: true,
              options: ASSIGNMENT_TYPES.map((t) => ({ label: t, value: t })),
            },
            { name: 'available_from', label: 'Available From', type: 'datetime', required: true },
            { name: 'due_date', label: 'Due Date', type: 'datetime', required: true },
            { name: 'description', label: 'Description', type: 'textarea' },
          ],
        }}
        columns={[
          {
            title: 'Name',
            dataIndex: 'name',
            key: 'name',
            sorter: { multiple: 1 },
          },
          {
            title: 'Type',
            dataIndex: 'assignment_type',
            key: 'assignment_type',
            sorter: { multiple: 2 },
            filters: ASSIGNMENT_TYPES.map((t) => ({ text: t, value: t })),
            render: (_, record) => <AssignmentTypeTag type={record.assignment_type} />,
          },
          {
            title: 'Available From',
            dataIndex: 'available_from',
            key: 'available_from',
            sorter: { multiple: 3 },
            render: (_, record) => dayjs(record.available_from).format('YYYY-MM-DD HH:mm'),
          },
          {
            title: 'Due Date',
            dataIndex: 'due_date',
            key: 'due_date',
            sorter: { multiple: 4 },
            render: (_, record) => dayjs(record.due_date).format('YYYY-MM-DD HH:mm'),
          },
          {
            title: 'Status',
            key: 'status',
            render: () => <AssignmentStatusTag status={getRandomAssignmentStatus()} />,
          },
        ]}
        sortOptions={[
          { label: 'Name', field: 'name' },
          { label: 'Type', field: 'assignment_type' },
          { label: 'Available From', field: 'available_from' },
          { label: 'Due Date', field: 'due_date' },
        ]}
        filterGroups={[
          {
            key: 'assignment_type',
            label: 'Type',
            type: 'select',
            options: ASSIGNMENT_TYPES.map((t) => ({ label: t, value: t })),
          },
          {
            key: 'name',
            label: 'Name',
            type: 'text',
          },
        ]}
        actions={
          <Button type="primary" onClick={() => setSetupOpen(true)}>
            Create Assignment
          </Button>
        }
      />

      <AssignmentSetup open={setupOpen} onClose={() => setSetupOpen(false)} module={module} />
    </div>
  );
};

export default AssignmentsList;
