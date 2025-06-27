import { Table, Typography, Tag, Button, Dropdown, Checkbox, message, Modal, Upload } from 'antd';
import { MoreOutlined, EyeOutlined, DownloadOutlined, UploadOutlined } from '@ant-design/icons';
import { useNavigate } from 'react-router-dom';
import type { ColumnsType } from 'antd/es/table';
import dayjs from 'dayjs';
import { useModule } from '@/context/ModuleContext';
import { useAssignment } from '@/context/AssignmentContext';
import { useEffect, useState } from 'react';
import { submitAssignment } from '@/services/modules/assignments/submissions/post';
import { getSubmissions } from '@/services/modules/assignments/submissions';
import type { Submission } from '@/types/modules/assignments/submissions';

const { Title, Text } = Typography;

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

const SubmissionsList = () => {
  const navigate = useNavigate();
  const module = useModule();
  const { assignment } = useAssignment();
  const [modalOpen, setModalOpen] = useState(false);
  const [selectedFile, setSelectedFile] = useState<File | null>(null);
  const [isPractice, setIsPractice] = useState(false);
  const [uploading, setUploading] = useState(false);
  const [submissions, setSubmissions] = useState<StudentSubmission[]>([]);
  const [loading, setLoading] = useState(false);

  const fetchSubmissions = async () => {
    if (!module.id || !assignment.id) return;
    setLoading(true);
    try {
      const query = new URLSearchParams({ page: '1', per_page: '50' });
      const res = await getSubmissions(module.id, assignment.id, query);
      let raw: Submission[] = [];

      if (Array.isArray(res.data)) {
        raw = res.data;
      } else if ('submissions' in res.data) {
        raw = res.data.submissions;
      }

      const items = raw.map(
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

      setSubmissions(items);
    } catch (err) {
      message.error('Failed to load submissions');
    } finally {
      setLoading(false);
    }
  };

  useEffect(() => {
    fetchSubmissions();
  }, [module.id, assignment.id]);

  const columns: ColumnsType<StudentSubmission> = [
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
    {
      title: 'Actions',
      key: 'actions',
      align: 'right',
      render: (_, record) => ({
        children: (
          <Dropdown
            trigger={['click']}
            menu={{
              items: [
                {
                  key: 'view',
                  icon: <EyeOutlined />,
                  label: 'View',
                },
                {
                  key: 'download',
                  icon: <DownloadOutlined />,
                  label: 'Download',
                },
              ],
              onClick: ({ key, domEvent }) => {
                domEvent.stopPropagation();
                if (key === 'view') {
                  navigate(
                    `/modules/${module.id}/assignments/${assignment.id}/submissions/${record.id}`,
                  );
                } else if (key === 'download') {
                  window.open(record.path, '_blank');
                }
              },
            }}
          >
            <Button icon={<MoreOutlined />} onClick={(e) => e.stopPropagation()} />
          </Dropdown>
        ),
      }),
    },
  ];

  return (
    <div className="max-w-4xl">
      <div className="flex items-center justify-between mb-2">
        <div>
          <Title level={4} className="mb-0">
            Your Submissions
          </Title>
          <Text className="text-gray-500 dark:text-gray-400">
            Below are your attempts for this assignment.
          </Text>
        </div>
        <Button type="primary" onClick={() => setModalOpen(true)}>
          New Submission
        </Button>
      </div>

      <Table<StudentSubmission>
        columns={columns}
        dataSource={submissions}
        rowKey="id"
        pagination={{ pageSize: 5 }}
        loading={loading}
        className="border-1 border-gray-200 dark:border-gray-800 rounded-lg overflow-hidden"
        onRow={(record) => ({
          onClick: () =>
            navigate(`/modules/${module.id}/assignments/${assignment.id}/submissions/${record.id}`),
        })}
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
            setSelectedFile(null);
            setIsPractice(false);
            await fetchSubmissions(); // <- re-fetch submissions after success
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
};

export default SubmissionsList;
