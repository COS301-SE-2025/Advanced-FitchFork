import { useState, useRef, useEffect } from 'react';
import { useNavigate } from 'react-router-dom';
import type { Announcement } from '@/types/modules/announcements';
import {
  listAnnouncements,
  createAnnouncement,
  updateAnnouncement,
  deleteAnnouncement,
} from '@/services/modules/announcements';
import { DeleteOutlined, EditOutlined, PlusOutlined } from '@ant-design/icons';
import PageHeader from '@/components/PageHeader';
import { EntityList, type EntityListHandle, type EntityListProps } from '@/components/EntityList';
import CreateModal from '@/components/common/CreateModal';
import EditModal from '@/components/common/EditModal';
import { message } from '@/utils/message';
import dayjs from 'dayjs';
import { useAuth } from '@/context/AuthContext';
import { Space, Typography } from 'antd';
import {
  AnnouncementListItem,
  AnnouncementsEmptyState,
  PinnedTag,
} from '@/components/announcements';
import AnnouncementCard from '@/components/announcements/AnnouncementCard';
import { useModule } from '@/context/ModuleContext';
import { mdExcerpt } from '@/utils/markdown';
import { useViewSlot } from '@/context/ViewSlotContext';

const Announcements = () => {
  const { setValue } = useViewSlot();
  const auth = useAuth();
  const navigate = useNavigate();
  const moduleDetails = useModule();
  const moduleId = moduleDetails.id;
  const listRef = useRef<EntityListHandle>(null);

  const [createOpen, setCreateOpen] = useState(false);
  const [editOpen, setEditOpen] = useState(false);
  const [editingItem, setEditingItem] = useState<Announcement | null>(null);

  const canManage = auth.isLecturer(moduleId) || auth.isAssistantLecturer(moduleId) || auth.isAdmin;
  const listMode =
    !auth.isLecturer(moduleId) && !auth.isAssistantLecturer(moduleId) && !auth.isAdmin;

  useEffect(() => {
    setValue(
      <Typography.Text className="text-base font-medium text-gray-900 dark:text-gray-100 truncate">
        Announcements
      </Typography.Text>,
    );
  }, []);

  const fetchAnnouncements = async ({
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
  }): Promise<{ items: Announcement[]; total: number }> => {
    const res = await listAnnouncements(moduleId, {
      page,
      per_page,
      query,
      pinned: filters.pinned?.[0] ? filters.pinned[0] === 'true' : undefined,
      sort,
    });

    if (res.success) {
      return {
        items: res.data.announcements,
        total: res.data.total,
      };
    } else {
      message.error(`Failed to fetch announcements: ${res.message}`);
      return { items: [], total: 0 };
    }
  };

  const handleCreate = async (values: Record<string, any>) => {
    const res = await createAnnouncement(moduleId, {
      title: values.title,
      body: values.body,
      pinned: Boolean(values.pinned),
    });

    if (res.success) {
      message.success(res.message || 'Announcement created');
      setCreateOpen(false);
      listRef.current?.refresh();
    } else {
      message.error(res.message);
    }
  };

  const handleEdit = async (values: Record<string, any>) => {
    if (!editingItem) return;

    const res = await updateAnnouncement(moduleId, editingItem.id, {
      title: values.title,
      body: values.body,
      pinned: Boolean(values.pinned),
    });

    if (res.success) {
      message.success(res.message || 'Announcement updated');
      setEditOpen(false);
      setEditingItem(null);
      listRef.current?.refresh();
    } else {
      message.error(res.message);
    }
  };

  const handleDelete = async (announcement: Announcement, refresh: () => void): Promise<void> => {
    const res = await deleteAnnouncement(moduleId, announcement.id);
    if (res.success) {
      message.success(res.message || 'Announcement deleted');
      refresh();
    } else {
      message.error(`Delete failed: ${res.message}`);
    }
  };

  const actions: EntityListProps<Announcement>['actions'] = canManage
    ? {
        control: [
          {
            key: 'create',
            label: 'Add Announcement',
            icon: <PlusOutlined />,
            isPrimary: true,
            handler: ({ refresh }) => {
              setCreateOpen(true);
              refresh();
            },
          },
        ],
        entity: (entity: Announcement) => [
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
      }
    : undefined;

  return (
    <div className="h-full flex flex-col overflow-hidden">
      <div className="flex-1 overflow-y-auto p-4">
        <Space direction="vertical" size="middle" className="w-full">
          <PageHeader
            title="Announcements"
            description={`All announcements for ${moduleDetails.code} (${moduleDetails.year})`}
          />

          <EntityList<Announcement>
            ref={listRef}
            name="Announcements"
            fetchItems={fetchAnnouncements}
            getRowKey={(a) => a.id}
            onRowClick={(a) => navigate(`/modules/${moduleId}/announcements/${a.id}`)}
            renderGridItem={(a, actions) => (
              <AnnouncementCard
                key={a.id}
                announcement={a}
                actions={actions}
                onClick={() => navigate(`/modules/${moduleId}/announcements/${a.id}`)} // NEW
              />
            )}
            listMode={listMode}
            renderListItem={(a) => (
              <AnnouncementListItem
                announcement={a}
                onClick={(ann) => navigate(`/modules/${moduleId}/announcements/${ann.id}`)}
              />
            )}
            columnToggleEnabled
            actions={actions}
            columns={[
              { title: 'ID', dataIndex: 'id', key: 'id', defaultHidden: true },
              { title: 'Title', dataIndex: 'title', key: 'title', sorter: { multiple: 1 } },
              {
                title: 'Pinned',
                dataIndex: 'pinned',
                key: 'pinned',
                sorter: { multiple: 2 },
                filters: [
                  { text: 'Pinned', value: 'true' },
                  { text: 'Unpinned', value: 'false' },
                ],
                render: (_, a) => <PinnedTag pinned={a.pinned} />,
              },
              {
                title: 'Body',
                dataIndex: 'body',
                key: 'body',
                defaultHidden: true,
                render: (_, a) => (
                  <div className="max-w-[48ch] line-clamp-2 text-gray-700 dark:text-neutral-300">
                    {mdExcerpt(a.body, 160)}
                  </div>
                ),
              },
              {
                title: 'Created At',
                dataIndex: 'created_at',
                key: 'created_at',
                sorter: { multiple: 3 },
                render: (_, a) => dayjs(a.created_at).format('YYYY-MM-DD HH:mm'),
              },
              {
                title: 'Updated At',
                dataIndex: 'updated_at',
                key: 'updated_at',
                sorter: { multiple: 4 },
                defaultHidden: true,
                render: (_, a) => dayjs(a.updated_at).format('YYYY-MM-DD HH:mm'),
              },
            ]}
            emptyNoEntities={
              <AnnouncementsEmptyState
                isLecturerOrAssistant={canManage}
                onCreate={() => setCreateOpen(true)}
              />
            }
          />
        </Space>

        <CreateModal
          open={createOpen}
          onCancel={() => setCreateOpen(false)}
          onCreate={handleCreate}
          initialValues={{ title: '', body: '', pinned: false }}
          fields={[
            { name: 'title', label: 'Title', type: 'text', required: true },
            { name: 'body', label: 'Body', type: 'textarea', required: true },
            { name: 'pinned', label: 'Pinned', type: 'boolean' },
          ]}
          title="Add Announcement"
        />

        <EditModal
          open={editOpen}
          onCancel={() => {
            setEditOpen(false);
            setEditingItem(null);
          }}
          onEdit={handleEdit}
          initialValues={{
            title: editingItem?.title ?? '',
            body: editingItem?.body ?? '',
            pinned: editingItem?.pinned ?? false,
          }}
          fields={[
            { name: 'title', label: 'Title', type: 'text', required: true },
            { name: 'body', label: 'Body', type: 'textarea', required: true },
            { name: 'pinned', label: 'Pinned', type: 'boolean' },
          ]}
          title="Edit Announcement"
        />
      </div>
    </div>
  );
};

export default Announcements;
