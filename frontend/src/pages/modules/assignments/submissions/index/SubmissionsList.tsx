import { Tag, Button, message, Upload, Checkbox, Modal, Input } from 'antd';
import { UploadOutlined } from '@ant-design/icons';
import { useNavigate } from 'react-router-dom';
import { useEffect, useRef, useState } from 'react';
import dayjs from 'dayjs';
import type { ColumnsType } from 'antd/es/table';

import { EntityList, type EntityListHandle } from '@/components/EntityList';
import { useModule } from '@/context/ModuleContext';
import { useAssignment } from '@/context/AssignmentContext';
import { getSubmissions } from '@/services/modules/assignments/submissions';
import { submitAssignment } from '@/services/modules/assignments/submissions/post';
import type { Submission } from '@/types/modules/assignments/submissions';
import type { SortOption } from '@/types/common';
import EventBus from '@/utils/EventBus';

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
  const entityListRef = useRef<EntityListHandle>(null);

  const [modalOpen, setModalOpen] = useState(false);
  const [selectedFile, setSelectedFile] = useState<File | null>(null);
  const [isPractice, setIsPractice] = useState(false);
  const [uploading, setUploading] = useState(false);

  useEffect(() => {
    setSelectedFile(null);
    setIsPractice(false);
  }, [modalOpen]);

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
    sort: SortOption[];
  }) => {
    if (!module.id || !assignment.id) {
      return { items: [], total: 0 };
    }

    const mappedSort = sort.map((s) => ({
      field: s.field,
      order: s.order,
    }));

    const res = await getSubmissions(module.id, assignment.id, {
      page,
      per_page,
      query,
      sort: mappedSort,
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

  const columns: ColumnsType<StudentSubmission> = [
    {
      title: 'Username',
      dataIndex: ['user', 'username'],
      key: 'user.username',
      sorter: { multiple: 1 },
      filters: [], // enable default funnel icon
      filterDropdown: ({ setSelectedKeys, selectedKeys, confirm, clearFilters }) => (
        <div style={{ padding: 8 }}>
          <Input
            placeholder="Search username"
            value={selectedKeys[0]}
            onChange={(e) => setSelectedKeys(e.target.value ? [e.target.value] : [])}
            onPressEnter={() => confirm()}
            style={{ width: 188, marginBottom: 8, display: 'block' }}
          />
          <div style={{ display: 'flex', gap: 8 }}>
            <Button type="primary" onClick={() => confirm()} size="small" style={{ width: 90 }}>
              Search
            </Button>
            <Button
              onClick={() => {
                clearFilters?.();
                confirm();
              }}
              size="small"
              style={{ width: 90 }}
            >
              Reset
            </Button>
          </div>
        </div>
      ),
    },

    {
      title: 'Attempt',
      dataIndex: 'attempt',
      key: 'attempt',
      sorter: { multiple: 2 },
      render: (attempt: number) => <Tag color="blue">#{attempt}</Tag>,
    },
    {
      title: 'Filename',
      dataIndex: 'filename',
      key: 'filename',
      sorter: { multiple: 3 },
    },
    {
      title: 'Submitted At',
      dataIndex: 'created_at',
      key: 'created_at',
      sorter: { multiple: 4 },
      render: (value: string) => dayjs(value).format('YYYY-MM-DD HH:mm'),
    },
    {
      title: 'Mark (%)',
      key: 'percentageMark',
      sorter: { multiple: 5 },
      render: (_, record: StudentSubmission) =>
        record.status === 'Graded' && typeof record.percentageMark === 'number' ? (
          <Tag color={getMarkColor(record.percentageMark)}>{record.percentageMark}%</Tag>
        ) : (
          <Tag color="default">Not marked</Tag>
        ),
    },
  ];

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
        filterGroups={[
          {
            key: 'user.username',
            label: 'Username',
            type: 'text',
          },
          {
            key: 'status',
            label: 'Status',
            type: 'select',
            options: [
              { label: 'Graded', value: 'Graded' },
              { label: 'Pending', value: 'Pending' },
            ],
          },
        ]}
        sortOptions={[
          { label: 'Username', field: 'username' },
          { label: 'Attempt', field: 'attempt' },
          { label: 'Filename', field: 'filename' },
          { label: 'Submitted At', field: 'created_at' },
          { label: 'Mark', field: 'mark' },
          { label: 'Status', field: 'status' },
        ]}
      />

      <Modal
        title="Submit Assignment"
        open={modalOpen}
        onCancel={() => setModalOpen(false)}
        onOk={async () => {
          if (!selectedFile) {
            message.error('Please select a file to submit.');
            return;
          }
          try {
            setUploading(true);
            await submitAssignment(module.id, assignment.id, selectedFile, isPractice);
            message.success('Submission successful', 3);
            setModalOpen(false);
          } catch (err) {
            message.error('Submission failed');
          } finally {
            setUploading(false);
          }
        }}
        okButtonProps={{ loading: uploading }}
        okText="Submit"
      >
        <Upload
          maxCount={1}
          beforeUpload={(file) => {
            setSelectedFile(file);
            return false;
          }}
          accept=".zip,.tar,.gz,.tgz"
        >
          <Button icon={<UploadOutlined />}>Click to select file</Button>
        </Upload>
        <Checkbox
          checked={isPractice}
          onChange={(e) => setIsPractice(e.target.checked)}
          style={{ marginTop: 16 }}
        >
          This is a practice submission
        </Checkbox>
      </Modal>
    </div>
  );
}
