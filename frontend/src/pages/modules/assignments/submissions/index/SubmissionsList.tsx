import { Tag, Button, message, Upload, Checkbox, Modal } from 'antd';
import { UploadOutlined } from '@ant-design/icons';
import { useNavigate } from 'react-router-dom';
import { useEffect, useState } from 'react';
import dayjs from 'dayjs';
import type { ColumnsType } from 'antd/es/table';

import { EntityList } from '@/components/EntityList';
import { useModule } from '@/context/ModuleContext';
import { useAssignment } from '@/context/AssignmentContext';
import { getSubmissions } from '@/services/modules/assignments/submissions';
import { submitAssignment } from '@/services/modules/assignments/submissions/post';
import type { Submission } from '@/types/modules/assignments/submissions';

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

  const [modalOpen, setModalOpen] = useState(false);
  const [selectedFile, setSelectedFile] = useState<File | null>(null);
  const [isPractice, setIsPractice] = useState(false);
  const [uploading, setUploading] = useState(false);
  const [hasUsername, setHasUsername] = useState(false);

  useEffect(() => {
    setSelectedFile(null);
    setIsPractice(false);
  }, [modalOpen]);

  const fetchItems = async ({ page, per_page }: { page: number; per_page: number }) => {
    if (!module.id || !assignment.id) {
      setHasUsername(false);
      return { items: [], total: 0 };
    }

    const query = new URLSearchParams({
      page: String(page),
      per_page: String(per_page),
    });

    const res = await getSubmissions(module.id, assignment.id, query);

    let raw: Submission[] = [];

    if (Array.isArray(res.data)) {
      raw = res.data;
    } else if ('submissions' in res.data) {
      raw = res.data.submissions;
    }

    const hasAnyUsername = raw.some((s) => s.user && s.user.username);
    setHasUsername(hasAnyUsername);

    const items: StudentSubmission[] = raw.map(
      (s): StudentSubmission => ({
        ...s,
        status: 'mark' in s ? 'Graded' : 'Pending',
        percentageMark:
          'mark' in s && s.mark && typeof s.mark === 'object' && 'earned' in s.mark
            ? Math.round(((s.mark as any).earned / (s.mark as any).total) * 100)
            : undefined,
        path: `/api/modules/${module.id}/assignments/${assignment.id}/submissions/${s.id}/file`,
      }),
    );

    return { items, total: items.length };
  };

  const columns: ColumnsType<StudentSubmission> = [
    ...(hasUsername
      ? [
          {
            title: 'Username',
            dataIndex: ['user', 'username'],
            key: 'username',
          },
        ]
      : []),
    {
      title: 'Attempt',
      dataIndex: 'attempt',
      key: 'attempt',
      render: (attempt) => <Tag color="blue">#{attempt}</Tag>,
    },
    {
      title: 'Filename',
      dataIndex: 'filename',
      key: 'filename',
    },
    {
      title: 'Submitted At',
      dataIndex: 'created_at',
      key: 'created_at',
      render: (value) => dayjs(value).format('YYYY-MM-DD HH:mm'),
    },
    {
      title: 'Status',
      dataIndex: 'status',
      key: 'status',
      render: (status) => <Tag color={status === 'Graded' ? 'green' : 'default'}>{status}</Tag>,
    },
    {
      title: 'Mark (%)',
      key: 'percentageMark',
      render: (_, record) =>
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
        name="Submissions"
        fetchItems={fetchItems}
        columns={columns}
        getRowKey={(item) => item.id}
        onRowClick={(item) =>
          navigate(`/modules/${module.id}/assignments/${assignment.id}/submissions/${item.id}`)
        }
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
