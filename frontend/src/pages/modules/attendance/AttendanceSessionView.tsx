// src/pages/modules/attendance/AttendanceSessionView.tsx
import { useEffect, useRef, useState } from 'react';
import { useNavigate, useParams } from 'react-router-dom';
import dayjs from 'dayjs';
import { Button, Card, Descriptions, Space, Tag, Typography } from 'antd';
import { EyeOutlined, ReloadOutlined, EditOutlined, DownloadOutlined } from '@ant-design/icons';

import PageHeader from '@/components/PageHeader';
import { message } from '@/utils/message';
import { useAuth } from '@/context/AuthContext';
import { useModule } from '@/context/ModuleContext';
import { useBreadcrumbContext } from '@/context/BreadcrumbContext';
import { useUI } from '@/context/UIContext';

import type {
  AttendanceRecord,
  AttendanceSession,
  EditAttendanceSessionReq,
} from '@/types/modules/attendance';
import {
  getAttendanceSession,
  listAttendanceSessionRecords,
  downloadAttendanceSessionRecordsCsv,
} from '@/services/modules/attendance/get';
import { editAttendanceSession } from '@/services/modules/attendance/put';

import EditModal from '@/components/common/EditModal';
import { EntityList, type EntityListHandle } from '@/components/EntityList';
import { IdTag } from '@/components/common';
import { useViewSlot } from '@/context/ViewSlotContext';

const fmt = (s: string) => dayjs(s).format('YYYY-MM-DD HH:mm');

export default function AttendanceSessionView() {
  const navigate = useNavigate();
  const { session_id } = useParams<{ session_id: string }>();

  const module = useModule();
  const moduleId = module.id;

  const auth = useAuth();
  const { setBreadcrumbLabel } = useBreadcrumbContext();
  const { isMobile } = useUI();
  const { setValue: setMobileHeader } = useViewSlot();

  const [loading, setLoading] = useState(true);
  const [session, setSession] = useState<AttendanceSession | null>(null);

  const [editOpen, setEditOpen] = useState(false);
  const [editDefaults, setEditDefaults] = useState<Record<string, any>>();

  const listRef = useRef<EntityListHandle>(null);

  const isStaff = auth.isAdmin || auth.hasLecturerRole?.() || auth.hasAssistantLecturerRole?.();

  // ── Load session
  const load = async () => {
    if (!session_id) return;
    setLoading(true);
    const res = await getAttendanceSession(moduleId, Number(session_id));
    setLoading(false);

    if (res.success && res.data) {
      const row = res.data;
      setSession(row);
      setBreadcrumbLabel(
        `modules/${moduleId}/attendance/sessions/${row.id}`,
        row.title || 'Session',
      );
    } else {
      message.error(res.message || 'Failed to load session');
    }
  };

  useEffect(() => {
    load();
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [moduleId, session_id]);

  useEffect(() => {
    const title = (session?.title || 'Attendance Session').trim();
    setMobileHeader(
      <Typography.Text className="text-base font-medium text-gray-900 dark:text-gray-100 truncate">
        {title}
      </Typography.Text>,
    );

    return () => setMobileHeader(null);
  }, []);

  useEffect(() => {
    if (session) {
      setBreadcrumbLabel(
        `modules/${moduleId}/attendance/sessions/${session.id}`,
        session.title || 'Session',
      );
    }
  }, [moduleId, session]);

  const openProjector = () =>
    navigate(`/modules/${moduleId}/attendance/sessions/${session_id}/projector`);

  const openEdit = () => {
    if (!session) return;
    setEditDefaults({
      title: session.title,
      active: session.active,
      rotation_seconds: session.rotation_seconds,
      restrict_by_ip: session.restrict_by_ip,
      allowed_ip_cidr: session.allowed_ip_cidr ?? '',
      created_from_ip: session.created_from_ip ?? '',
    });
    setEditOpen(true);
  };

  const handleEdit = async (values: Record<string, any>) => {
    if (!session) return;

    // returns a trimmed string or undefined (never `unknown`)
    const strOrUndef = (v: unknown): string | undefined =>
      typeof v === 'string' && v.trim() ? v.trim() : undefined;

    const payload: EditAttendanceSessionReq = {
      title: strOrUndef(values.title),
      active: typeof values.active === 'boolean' ? values.active : undefined,
      rotation_seconds:
        values.rotation_seconds !== undefined ? Number(values.rotation_seconds) : undefined,
      restrict_by_ip:
        typeof values.restrict_by_ip === 'boolean' ? values.restrict_by_ip : undefined,
      allowed_ip_cidr: strOrUndef(values.allowed_ip_cidr),
      created_from_ip: strOrUndef(values.created_from_ip),
    };

    const res = await editAttendanceSession(moduleId, session.id, payload);

    if (res.success) {
      message.success(res.message || 'Attendance session updated');
      setEditOpen(false);
      setEditDefaults(undefined);
      await load();
      // refresh the records list if visible
      if (!isMobile) {
        listRef.current?.refresh();
      }
    } else {
      message.error(res.message || 'Failed to update session');
    }
  };

  const handleExportCsv = async () => {
    if (!session) return;
    try {
      await downloadAttendanceSessionRecordsCsv(moduleId, session.id);
    } catch (e: any) {
      message.error(e?.message || 'Export failed');
    }
  };

  // ── Records data source for EntityList (left pane)
  const fetchRecords = async ({
    page,
    per_page,
    query,
    sort,
  }: {
    page: number;
    per_page: number;
    query?: string;
    sort: { field: string; order: 'ascend' | 'descend' }[];
  }): Promise<{ items: AttendanceRecord[]; total: number }> => {
    if (!session_id) return { items: [], total: 0 };

    const res = await listAttendanceSessionRecords(moduleId, Number(session_id), {
      page,
      per_page,
      q: query,
      sort, // buildQuery will serialize to a single sort string
    });

    if (res.success) {
      return {
        items: res.data.records,
        total: res.data.total,
      };
    } else {
      message.error(res.message || 'Failed to fetch records');
      return { items: [], total: 0 };
    }
  };

  const hasCidr = !!session?.allowed_ip_cidr && session.allowed_ip_cidr.trim().length > 0;
  const hasCreatorIp = !!session?.created_from_ip && session.created_from_ip.trim().length > 0;

  return (
    <div className="h-full flex flex-col overflow-hidden">
      <div className="flex-1 overflow-y-auto p-4 space-y-4">
        <PageHeader
          title={session?.title ?? 'Attendance Session'}
          description={<span className="block mt-1">Record log and session details.</span>}
          extra={
            <Space wrap>
              <Button
                icon={<ReloadOutlined />}
                onClick={() => {
                  load();
                  if (!isMobile) listRef.current?.refresh();
                }}
              />
              {isStaff && (
                <>
                  <Button icon={<EditOutlined />} onClick={openEdit} disabled={!session}>
                    Edit
                  </Button>
                  <Button
                    type="default"
                    icon={<DownloadOutlined />}
                    onClick={handleExportCsv}
                    disabled={!session}
                  >
                    Export CSV
                  </Button>
                  <Button
                    type="primary"
                    icon={<EyeOutlined />}
                    onClick={openProjector}
                    disabled={!session}
                  >
                    Projector
                  </Button>
                </>
              )}
            </Space>
          }
        />

        {/* Layout: on mobile we ONLY show the details card; on larger screens, show 2-pane */}
        <div
          className={isMobile ? 'grid grid-cols-1 gap-4' : 'grid grid-cols-1 lg:grid-cols-3 gap-4'}
        >
          {/* LEFT (hidden on mobile): Records list */}
          {!isMobile && (
            <div className="lg:col-span-2 min-w-0">
              <EntityList<AttendanceRecord>
                ref={listRef}
                name="Attendance Records"
                fetchItems={fetchRecords}
                getRowKey={(r) => `${r.session_id}:${r.user_id}:${r.token_window}`}
                listMode={false}
                columnToggleEnabled
                columns={[
                  {
                    title: 'User ID',
                    dataIndex: 'user_id',
                    key: 'user_id',
                    sorter: { multiple: 1 },
                    width: 110,
                    render: (id) => <IdTag id={id} />,
                  },
                  {
                    title: 'Username',
                    dataIndex: 'username',
                    key: 'username',
                    ellipsis: true,
                  },
                  {
                    title: 'Taken At',
                    dataIndex: 'taken_at',
                    key: 'taken_at',
                    sorter: { multiple: 2 },
                    render: (_, r) => dayjs(r.taken_at).format('YYYY-MM-DD HH:mm:ss'),
                    width: 190,
                  },
                  {
                    title: 'IP Address',
                    dataIndex: 'ip_address',
                    key: 'ip_address',
                    width: 160,
                  },
                  {
                    title: 'Window',
                    dataIndex: 'token_window',
                    key: 'token_window',
                    width: 120,
                  },
                ]}
                emptyNoEntities={
                  <div className="p-8 text-center text-sm text-gray-500">No records yet.</div>
                }
              />
            </div>
          )}

          {/* RIGHT: Session details */}
          <div className={isMobile ? 'col-span-1' : 'lg:col-span-1 min-w-0'}>
            <Card loading={loading} title="Session details">
              {session && (
                <Descriptions bordered column={1} size="middle">
                  <Descriptions.Item label="Status">
                    {session.active ? <Tag color="green">Active</Tag> : <Tag>Inactive</Tag>}
                  </Descriptions.Item>
                  <Descriptions.Item label="Rotation">
                    {session.rotation_seconds}s
                  </Descriptions.Item>
                  <Descriptions.Item label="IP Restricted">
                    {session.restrict_by_ip ? 'Yes' : 'No'}
                  </Descriptions.Item>
                  {hasCidr && (
                    <Descriptions.Item label="Allowed CIDR">
                      {session.allowed_ip_cidr}
                    </Descriptions.Item>
                  )}
                  {hasCreatorIp && (
                    <Descriptions.Item label="Creator IP">
                      {session.created_from_ip}
                    </Descriptions.Item>
                  )}
                  <Descriptions.Item label="Attended / Students">
                    {session.attended_count} / {session.student_count}
                  </Descriptions.Item>
                  <Descriptions.Item label="Created">{fmt(session.created_at)}</Descriptions.Item>
                  <Descriptions.Item label="Updated">{fmt(session.updated_at)}</Descriptions.Item>
                </Descriptions>
              )}
            </Card>
          </div>
        </div>
      </div>

      {/* Edit modal */}
      <EditModal
        open={editOpen}
        onCancel={() => {
          setEditOpen(false);
          setEditDefaults(undefined);
        }}
        onEdit={handleEdit}
        title="Edit Attendance Session"
        initialValues={editDefaults ?? {}}
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
  );
}
