import { useEffect, useRef, useState } from 'react';
import { PlusOutlined, DeleteOutlined, LockOutlined, UnlockOutlined } from '@ant-design/icons';
import dayjs from 'dayjs';
import { useAssignment } from '@/context/AssignmentContext';
import { useAuth } from '@/context/AuthContext';
import { useModule } from '@/context/ModuleContext';
import {
  TICKET_STATUSES,
  type Ticket,
  type TicketStatus,
} from '@/types/modules/assignments/tickets';
import type { SortOption } from '@/types/common';
import { type EntityListHandle, EntityList } from '@/components/EntityList';
import FormModal, { type FormModalField } from '@/components/common/FormModal';
import { message } from '@/utils/message';
import { deleteTicket } from '@/services/modules/assignments/tickets/delete';
import { listTickets } from '@/services/modules/assignments/tickets/get';
import { createTicket } from '@/services/modules/assignments/tickets/post';
import { openTicket, closeTicket } from '@/services/modules/assignments/tickets/put';
import TicketStatusTag from '@/components/tickets/TicketStatusTag';
import TicketCard from '@/components/tickets/TicketCard';
import { useNavigate } from 'react-router-dom';
import { Typography } from 'antd';
import { useViewSlot } from '@/context/ViewSlotContext';
import TicketListItem from '@/components/tickets/TicketListItem';
import { TicketsEmptyState } from '@/components/tickets';

const ticketFields: FormModalField[] = [
  {
    name: 'title',
    label: 'Title',
    type: 'text',
    constraints: { required: true, length: { min: 3, max: 160 } },
  },
  {
    name: 'description',
    label: 'Description',
    type: 'textarea',
    constraints: { length: { max: 4000 } },
    // optional UI hints:
    ui: { props: { rows: 4, showCount: true, maxLength: 4000 } },
  },
];

const Tickets = () => {
  const auth = useAuth();
  const module = useModule();
  const { assignment } = useAssignment();
  const navigate = useNavigate();
  const { setValue } = useViewSlot();

  const listRef = useRef<EntityListHandle>(null);

  const [createOpen, setCreateOpen] = useState(false);

  const isAdminOrLecturer = auth.isAdmin || auth.isLecturer(module.id);
  const isStudent = auth.isStudent(module.id);

  useEffect(() => {
    setValue(
      <Typography.Text className="text-base font-medium text-gray-900 dark:text-gray-100">
        Tickets
      </Typography.Text>,
    );
  }, []);
  const fetchTickets = async ({
    page,
    per_page,
    sort,
    filters,
    query,
  }: {
    page: number;
    per_page: number;
    sort: SortOption[];
    filters: Record<string, string[]>;
    query?: string;
  }): Promise<{ items: Ticket[]; total: number }> => {
    const res = await listTickets(module.id, assignment.id, {
      page,
      per_page,
      query,
      status: filters.status?.[0] as TicketStatus | undefined,
      sort,
    });

    if (res.success) {
      return {
        items: res.data.tickets,
        total: res.data.total,
      };
    } else {
      message.error(`Failed to fetch tickets: ${res.message}`);
      return { items: [], total: 0 };
    }
  };

  const handleCreate = async (values: Record<string, any>) => {
    const res = await createTicket(module.id, assignment.id, {
      title: values.title,
      description: values.description,
      status: 'open',
    });

    if (res.success) {
      message.success('Ticket created');
      setCreateOpen(false);
      listRef.current?.refresh();
    } else {
      message.error(res.message);
    }
  };

  const handleDelete = async (ticket: Ticket, refresh: () => void) => {
    const res = await deleteTicket(module.id, assignment.id, ticket.id);
    if (res.success) {
      message.success('Ticket deleted');
      refresh();
    } else {
      message.error(res.message);
    }
  };

  return (
    <>
      <EntityList<Ticket>
        ref={listRef}
        name="Tickets"
        defaultViewMode="table"
        listMode={isStudent}
        fetchItems={fetchTickets}
        getRowKey={(t) => t.id}
        onRowClick={(t) =>
          navigate(`/modules/${module.id}/assignments/${assignment.id}/tickets/${t.id}`)
        }
        renderGridItem={(ticket, actions) => (
          <TicketCard
            key={ticket.id}
            ticket={ticket}
            actions={actions}
            onClick={(t) =>
              navigate(`/modules/${module.id}/assignments/${assignment.id}/tickets/${t.id}`)
            }
          />
        )}
        renderListItem={(ticket) => (
          <TicketListItem
            ticket={ticket}
            onClick={(t) =>
              navigate(`/modules/${module.id}/assignments/${assignment.id}/tickets/${t.id}`)
            }
          />
        )}
        columnToggleEnabled
        columns={[
          { title: 'ID', dataIndex: 'id', key: 'id', defaultHidden: true },
          { title: 'Title', dataIndex: 'title', key: 'title' },
          {
            title: 'Description',
            dataIndex: 'description',
            key: 'description',
            defaultHidden: true,
          },
          {
            title: 'Status',
            dataIndex: 'status',
            key: 'status',
            sorter: { multiple: 1 },
            filters: TICKET_STATUSES.map((s) => ({ text: s, value: s })),
            render: (v) => <TicketStatusTag status={v} />,
          },
          {
            title: 'Created At',
            dataIndex: 'created_at',
            key: 'created_at',
            sorter: { multiple: 2 },
            render: (_, r) => dayjs(r.created_at).format('YYYY-MM-DD HH:mm'),
          },

          {
            title: 'Updated At',
            dataIndex: 'updated_at',
            key: 'updated_at',
            sorter: { multiple: 3 },
            render: (_, r) => dayjs(r.updated_at).format('YYYY-MM-DD HH:mm'),
          },
        ]}
        actions={
          isAdminOrLecturer || isStudent
            ? {
                control: [
                  {
                    key: 'create',
                    label: 'New Ticket',
                    icon: <PlusOutlined />,
                    isPrimary: true,
                    handler: () => setCreateOpen(true),
                  },
                ],
                entity: (ticket) => {
                  const actions = [];

                  if (ticket.status === 'closed') {
                    actions.push({
                      key: 'open',
                      label: 'Open',
                      icon: <UnlockOutlined />,
                      confirm: false,
                      handler: async ({ refresh }: { refresh: () => void }) => {
                        const res = await openTicket(module.id, assignment.id, ticket.id);
                        res.success
                          ? message.success('Ticket opened')
                          : message.error(res.message || 'Failed to open ticket');
                        refresh();
                      },
                    });
                  }

                  if (ticket.status === 'open') {
                    actions.push({
                      key: 'close',
                      label: 'Close',
                      icon: <LockOutlined />,
                      confirm: false,
                      handler: async ({ refresh }: { refresh: () => void }) => {
                        const res = await closeTicket(module.id, assignment.id, ticket.id);
                        res.success
                          ? message.success('Ticket closed')
                          : message.error(res.message || 'Failed to close ticket');
                        refresh();
                      },
                    });
                  }

                  actions.push({
                    key: 'delete',
                    label: 'Delete',
                    icon: <DeleteOutlined />,
                    confirm: true,
                    handler: ({ refresh }: { refresh: () => void }) =>
                      handleDelete(ticket, refresh),
                  });
                  return actions;
                },
              }
            : undefined
        }
        emptyNoEntities={
          <TicketsEmptyState
            onCreate={() => setCreateOpen(true)}
            onRefresh={() => listRef.current?.refresh()}
          />
        }
        filtersStorageKey={`modules:${module.id}:assignments:${assignment.id}:tickets:filters:v1`}
      />

      <FormModal
        open={createOpen}
        onCancel={() => setCreateOpen(false)}
        onSubmit={handleCreate}
        title="Create Ticket"
        submitText="Create"
        initialValues={{ title: '', description: '' }}
        fields={ticketFields}
      />
    </>
  );
};

export default Tickets;
