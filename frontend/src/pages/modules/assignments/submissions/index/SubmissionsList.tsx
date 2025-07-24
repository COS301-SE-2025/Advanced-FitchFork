import { Tag } from 'antd';
import { DeleteOutlined, ReloadOutlined } from '@ant-design/icons';
import { useNavigate } from 'react-router-dom';
import { useEffect, useRef } from 'react';
import dayjs from 'dayjs';

import {
  EntityList,
  type EntityListHandle,
  type EntityColumnType,
  type EntityAction,
  type EntityListProps,
} from '@/components/EntityList';
import { useModule } from '@/context/ModuleContext';
import { useAssignment } from '@/context/AssignmentContext';
import { useAuth } from '@/context/AuthContext';
import { getSubmissions } from '@/services/modules/assignments/submissions';
import EventBus from '@/utils/EventBus';

import type { Submission } from '@/types/modules/assignments/submissions';
import SubmissionCard from '@/components/submissions/SubmissionCard';
import { message } from '@/utils/message';

const getMarkColor = (mark: number): string => {
  if (mark >= 75) return 'green';
  if (mark >= 50) return 'orange';
  return 'red';
};

type StudentSubmission = Submission & {
  status: 'Pending' | 'Graded';
  path: string;
  percentageMark?: number;
};

export default function SubmissionsList() {
  const navigate = useNavigate();
  const module = useModule();
  const { assignment } = useAssignment();
  const auth = useAuth();

  const entityListRef = useRef<EntityListHandle>(null);

  useEffect(() => {
    const listener = () => {
      entityListRef.current?.refresh();
    };
    EventBus.on('submission:updated', listener);

    return () => {
      EventBus.off('submission:updated', listener);
    };
  }, []);

  const fetchItems = async ({
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
  }) => {
    if (!module.id || !assignment.id) {
      return { items: [], total: 0 };
    }

    const res = await getSubmissions(module.id, assignment.id, {
      page,
      per_page,
      query,
      sort,
      username: filters['user.username']?.[0],
      status: filters['status']?.[0],
    });

    const { submissions, total } = res.data;

    const items: StudentSubmission[] = submissions.map(
      (s): StudentSubmission => ({
        ...s,
        status: s.mark ? 'Graded' : 'Pending',
        percentageMark:
          s.mark && typeof s.mark === 'object' && 'earned' in s.mark
            ? Math.round(((s.mark as any).earned / (s.mark as any).total) * 100)
            : undefined,
        path: `/api/modules/${module.id}/assignments/${assignment.id}/submissions/${s.id}/file`,
      }),
    );

    return { items, total };
  };

  const columns: EntityColumnType<StudentSubmission>[] = [
    { title: 'ID', dataIndex: 'id', key: 'id', defaultHidden: true },
    {
      title: 'Username',
      dataIndex: ['user', 'username'],
      key: 'user.username',
      sorter: { multiple: 1 },
    },
    {
      title: 'Attempt',
      dataIndex: 'attempt',
      key: 'attempt',
      sorter: { multiple: 2 },
      render: (attempt) => <Tag color="blue">#{attempt}</Tag>,
    },
    { title: 'Filename', dataIndex: 'filename', key: 'filename', defaultHidden: true },
    {
      title: 'Is Practice',
      dataIndex: 'is_practice',
      key: 'is_practice',
      defaultHidden: true,
      render: (val) => (val ? <Tag color="gold">Yes</Tag> : <Tag>No</Tag>),
    },
    {
      title: 'Mark (%)',
      key: 'percentageMark',
      sorter: { multiple: 3 },
      render: (_, record) =>
        record.status === 'Graded' && typeof record.percentageMark === 'number' ? (
          <Tag color={getMarkColor(record.percentageMark)}>{record.percentageMark}%</Tag>
        ) : (
          <Tag color="default">Not marked</Tag>
        ),
    },
    {
      title: 'Is Late',
      dataIndex: 'is_late',
      key: 'is_late',
      defaultHidden: true,
      render: (val) => (val ? <Tag color="red">Yes</Tag> : <Tag>On Time</Tag>),
    },
    {
      title: 'Created At',
      dataIndex: 'created_at',
      key: 'created_at',
      defaultHidden: true,
      render: (value) => dayjs(value).format('YYYY-MM-DD HH:mm'),
    },
    {
      title: 'Updated At',
      dataIndex: 'updated_at',
      key: 'updated_at',
      defaultHidden: true,
      render: (value) => dayjs(value).format('YYYY-MM-DD HH:mm'),
    },
  ];

  const canManageSubmissions = auth.isLecturer(module.id) || auth.isAdmin;

  const actions: EntityListProps<StudentSubmission>['actions'] = canManageSubmissions
    ? {
        entity: (entity: StudentSubmission): EntityAction<StudentSubmission>[] => [
          {
            key: 'delete',
            label: 'Delete',
            icon: <DeleteOutlined />,
            handler: ({ refresh }) => {
              message.success(`Deleted submission ${entity.id}`);
              refresh();
            },
          },
          {
            key: 'remark',
            label: 'Re-mark',
            icon: <ReloadOutlined />,
            handler: ({ refresh }) => {
              message.success(`Re-marked submission ${entity.id}`);
              refresh();
            },
          },
          {
            key: 'remark2',
            label: 'Re-mark',
            icon: <ReloadOutlined />,
            handler: ({ refresh }) => {
              message.success(`Re-marked submission ${entity.id}`);
              refresh();
            },
          },
          {
            key: 'remark3',
            label: 'Re-mark',
            icon: <ReloadOutlined />,
            handler: ({ refresh }) => {
              message.success(`Re-marked submission ${entity.id}`);
              refresh();
            },
          },
        ],
        bulk: [
          {
            key: 'bulk-delete',
            label: 'Bulk Delete',
            icon: <DeleteOutlined />,
            handler: ({ selected, refresh }) => {
              message.success(`Deleted ${selected?.length || 0} submissions`);
              refresh();
            },
          },
          {
            key: 'bulk-remark',
            label: 'Bulk Re-mark',
            icon: <ReloadOutlined />,
            handler: ({ selected, refresh }) => {
              message.success(`Re-marked ${selected?.length || 0} submissions`);
              refresh();
            },
          },
        ],
      }
    : undefined;

  return (
    <div>
      <EntityList<StudentSubmission>
        ref={entityListRef}
        name="Submissions"
        fetchItems={fetchItems}
        columns={columns}
        getRowKey={(item) => item.id}
        onRowClick={(item) =>
          navigate(`/modules/${module.id}/assignments/${assignment.id}/submissions/${item.id}`)
        }
        columnToggleEnabled
        actions={actions}
        renderGridItem={(item, itemActions) => (
          <SubmissionCard submission={item} actions={itemActions} />
        )}
      />
    </div>
  );
}
