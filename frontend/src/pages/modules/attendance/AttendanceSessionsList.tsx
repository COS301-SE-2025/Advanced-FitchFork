// src/pages/modules/attendance/AttendanceSessionsList.tsx
import { useRef, useState } from 'react';
import { useNavigate } from 'react-router-dom';
import dayjs from 'dayjs';
import { Tag, Switch, Progress, Tooltip } from 'antd';
import { DeleteOutlined, EditOutlined, EyeOutlined, PlusOutlined } from '@ant-design/icons';

import type { AttendanceSession } from '@/types/modules/attendance';
import PageHeader from '@/components/PageHeader';
import { EntityList, type EntityListHandle, type EntityListProps } from '@/components/EntityList';
import CreateModal from '@/components/common/CreateModal';
import EditModal from '@/components/common/EditModal';
import { message } from '@/utils/message';
import { useAuth } from '@/context/AuthContext';
import { useModule } from '@/context/ModuleContext';

import { listAttendanceSessions } from '@/services/modules/attendance/get';
import { createAttendanceSession } from '@/services/modules/attendance/post';
import { editAttendanceSession } from '@/services/modules/attendance/put';
import { deleteAttendanceSession } from '@/services/modules/attendance/delete';
import { AttendanceEmptyState, AttendanceSessionCard } from '@/components/attendance';

const fmt = (s: string) => dayjs(s).format('YYYY-MM-DD HH:mm');

const AttendanceSessionsList = () => {
  const navigate = useNavigate();
  const auth = useAuth();
  const { id: moduleId } = useModule();

  const listRef = useRef<EntityListHandle>(null);

  const [createOpen, setCreateOpen] = useState(false);
  const [editOpen, setEditOpen] = useState(false);
  const [editingItem, setEditingItem] = useState<AttendanceSession | null>(null);

  const isStaff = auth.isAdmin || auth.hasLecturerRole?.() || auth.hasAssistantLecturerRole?.();

  // ---- Data source
  const fetchSessions = async ({
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
  }): Promise<{ items: AttendanceSession[]; total: number }> => {
    const res = await listAttendanceSessions(moduleId, {
      page,
      per_page,
      q: query,
      active:
        filters.active?.[0] === 'true' ? true : filters.active?.[0] === 'false' ? false : undefined,
      sort,
    });

    if (res.success) {
      return {
        items: res.data.sessions,
        total: res.data.total,
      };
    } else {
      message.error(`Failed to fetch attendance sessions: ${res.message}`);
      return { items: [], total: 0 };
    }
  };

  // ---- Handlers
  const handleCreate = async (values: Record<string, any>) => {
    const res = await createAttendanceSession(moduleId, {
      title: values.title,
      active: !!values.active,
      rotation_seconds: Number(values.rotation_seconds),
      restrict_by_ip: !!values.restrict_by_ip,
      allowed_ip_cidr: values.allowed_ip_cidr || undefined,
      pin_to_creator_ip: !!values.pin_to_creator_ip,
    });

    if (res.success) {
      message.success(res.message || 'Attendance session created');
      setCreateOpen(false);
      listRef.current?.refresh();
    } else {
      message.error(res.message);
    }
  };

  const handleEdit = async (values: Record<string, any>) => {
    if (!editingItem) return;
    const res = await editAttendanceSession(moduleId, editingItem.id, {
      title: values.title,
      active: values.active !== undefined ? !!values.active : undefined,
      rotation_seconds:
        values.rotation_seconds !== undefined ? Number(values.rotation_seconds) : undefined,
      restrict_by_ip: values.restrict_by_ip !== undefined ? !!values.restrict_by_ip : undefined,
      allowed_ip_cidr: values.allowed_ip_cidr ?? undefined,
      created_from_ip: values.created_from_ip ?? undefined,
    });

    if (res.success) {
      message.success(res.message || 'Attendance session updated');
      setEditOpen(false);
      setEditingItem(null);
      listRef.current?.refresh();
    } else {
      message.error(res.message);
    }
  };

  const handleDelete = async (sess: AttendanceSession, refresh: () => void) => {
    const res = await deleteAttendanceSession(moduleId, sess.id);
    if (res.success) {
      message.success(res.message || 'Attendance session deleted');
      refresh();
    } else {
      message.error(`Delete failed: ${res.message}`);
    }
  };

  // ---- Actions (reused by table & card)
  const actions: EntityListProps<AttendanceSession>['actions'] | undefined = isStaff
    ? {
        control: [
          {
            key: 'create',
            label: 'New Session',
            icon: <PlusOutlined />,
            isPrimary: true,
            handler: () => setCreateOpen(true),
          },
        ],
        entity: (s) => [
          {
            key: 'projector',
            label: 'Projector Mode',
            icon: <EyeOutlined />,
            handler: () => {
              navigate(`/modules/${moduleId}/attendance/sessions/${s.id}/projector`);
            },
          },
          {
            key: 'edit',
            label: 'Edit',
            icon: <EditOutlined />,
            handler: () => {
              setEditingItem(s);
              setEditOpen(true);
            },
          },
          {
            key: 'delete',
            label: 'Delete',
            icon: <DeleteOutlined />,
            confirm: true,
            handler: ({ refresh }) => handleDelete(s, refresh),
          },
        ],
      }
    : undefined;

  return (
    <div className="h-full flex flex-col overflow-hidden">
      <div className="flex-1 overflow-y-auto p-4">
        <div className="flex h-full flex-col gap-4">
          <PageHeader
            title="Attendance Sessions"
            description="Create and manage rotating-code attendance sessions for this module."
          />

          <EntityList<AttendanceSession>
            ref={listRef}
            name="Attendance Sessions"
            fetchItems={fetchSessions}
            getRowKey={(s) => s.id}
            onRowClick={(s) => navigate(`/modules/${moduleId}/attendance/sessions/${s.id}`)}
            /** Show a clean grid of cards to non-staff; staff default to table for bulk work */
            defaultViewMode={isStaff ? 'table' : 'grid'}
            renderGridItem={(s, cardActions) => (
              <AttendanceSessionCard
                key={s.id}
                session={s}
                actions={cardActions}
                onClick={() => navigate(`/modules/${moduleId}/attendance/sessions/${s.id}`)}
              />
            )}
            columnToggleEnabled
            actions={actions}
            /** Table columns remain for staff/table mode */
            columns={[
              { title: 'ID', dataIndex: 'id', key: 'id', defaultHidden: true },
              {
                title: 'Title',
                dataIndex: 'title',
                key: 'title',
                sorter: { multiple: 1 },
              },
              {
                title: 'Attend',
                key: 'attend',
                render: (_, s) => {
                  const total = s.student_count ?? 0;
                  const attended = s.attended_count ?? 0;
                  const pct = total > 0 ? Math.round((attended / total) * 100) : 0;
                  const strokeColor = pct >= 75 ? '#52c41a' : pct >= 40 ? '#faad14' : '#ff4d4f';

                  const tip = (
                    <div className="text-sm">
                      <div>
                        <strong>Attended:</strong> {attended}
                      </div>
                      <div>
                        <strong>Total students:</strong> {total}
                      </div>
                      <div>
                        <strong>Percentage:</strong> {pct}%
                      </div>
                      <div>
                        <strong>Status:</strong> {s.active ? 'Active session' : 'Inactive session'}
                      </div>
                    </div>
                  );

                  return (
                    <Tooltip title={tip}>
                      <div style={{ minWidth: 160 }}>
                        <Progress
                          percent={pct}
                          size="small"
                          status={s.active ? 'active' : undefined}
                          strokeColor={strokeColor}
                          aria-label={`Attendance ${pct} percent (${attended} of ${total})`}
                        />
                      </div>
                    </Tooltip>
                  );
                },
              },
              {
                title: 'Active',
                dataIndex: 'active',
                key: 'active',
                filters: [
                  { text: 'Active', value: 'true' },
                  { text: 'Inactive', value: 'false' },
                ],
                render: (_, s) =>
                  s.active ? <Tag color="success">Active</Tag> : <Tag>Inactive</Tag>,
              },
              {
                title: 'Rotation (s)',
                dataIndex: 'rotation_seconds',
                key: 'rotation_seconds',
                sorter: { multiple: 2 },
                defaultHidden: true,
              },
              {
                title: 'IP Restricted',
                dataIndex: 'restrict_by_ip',
                key: 'restrict_by_ip',
                sorter: { multiple: 3 },
                render: (_, s) => <Switch size="small" checked={s.restrict_by_ip} disabled />,
                defaultHidden: true,
              },
              {
                title: 'Allowed CIDR',
                dataIndex: 'allowed_ip_cidr',
                key: 'allowed_ip_cidr',
                defaultHidden: true,
              },
              {
                title: 'Creator IP',
                dataIndex: 'created_from_ip',
                key: 'created_from_ip',
                defaultHidden: true,
              },
              {
                title: 'Created',
                dataIndex: 'created_at',
                key: 'created_at',
                render: (_, s) => fmt(s.created_at),
                defaultHidden: true,
              },
              {
                title: 'Updated',
                dataIndex: 'updated_at',
                key: 'updated_at',
                render: (_, s) => fmt(s.updated_at),
                defaultHidden: true,
              },
            ]}
            emptyNoEntities={
              <AttendanceEmptyState
                isStaff={isStaff}
                onCreate={isStaff ? () => setCreateOpen(true) : undefined}
                onRefresh={() => listRef.current?.refresh()}
              />
            }
          />
        </div>

        {/* Create */}
        <CreateModal
          open={createOpen}
          onCancel={() => setCreateOpen(false)}
          onCreate={handleCreate}
          title="Create Attendance Session"
          initialValues={{
            title: '',
            active: true,
            rotation_seconds: 30,
            restrict_by_ip: false,
            allowed_ip_cidr: '',
            pin_to_creator_ip: false,
          }}
          fields={[
            { name: 'title', label: 'Title', type: 'text', required: true },
            { name: 'active', label: 'Enabled', type: 'boolean' },
            {
              name: 'rotation_seconds',
              label: 'Rotation (seconds)',
              type: 'number',
              required: true,
            },
            { name: 'restrict_by_ip', label: 'Restrict by IP', type: 'boolean' },
            { name: 'allowed_ip_cidr', label: 'Allowed CIDR (optional)', type: 'text' },
            { name: 'pin_to_creator_ip', label: 'Pin to my current IP', type: 'boolean' },
          ]}
        />

        {/* Edit */}
        <EditModal
          open={editOpen}
          onCancel={() => {
            setEditOpen(false);
            setEditingItem(null);
          }}
          onEdit={handleEdit}
          title="Edit Attendance Session"
          initialValues={{
            title: editingItem?.title ?? '',
            active: editingItem?.active,
            rotation_seconds: editingItem?.rotation_seconds,
            restrict_by_ip: editingItem?.restrict_by_ip,
            allowed_ip_cidr: editingItem?.allowed_ip_cidr ?? '',
            created_from_ip: editingItem?.created_from_ip ?? '',
          }}
          fields={[
            { name: 'title', label: 'Title', type: 'text' },
            { name: 'active', label: 'Enabled', type: 'boolean' },
            { name: 'rotation_seconds', label: 'Rotation (seconds)', type: 'number' },
            { name: 'restrict_by_ip', label: 'Restrict by IP', type: 'boolean' },
            { name: 'allowed_ip_cidr', label: 'Allowed CIDR', type: 'text' },
            { name: 'created_from_ip', label: 'Creator IP', type: 'text' },
          ]}
        />
      </div>
    </div>
  );
};

export default AttendanceSessionsList;
