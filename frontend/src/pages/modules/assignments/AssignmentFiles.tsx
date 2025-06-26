import { useEffect, useMemo, useState } from 'react';
import { Segmented, Upload, Typography, message, Table, Button, Space, Tooltip } from 'antd';
import {
  UploadOutlined,
  DownloadOutlined,
  ReloadOutlined,
  FileTextOutlined,
  FileAddOutlined,
  FileMarkdownOutlined,
  FileZipOutlined,
  SettingOutlined,
  CodeOutlined,
} from '@ant-design/icons';

import { useAssignment } from '@/context/AssignmentContext';
import { useModule } from '@/context/ModuleContext';
import { downloadAssignmentFile, uploadAssignmentFile } from '@/services/modules/assignments';

import type { AssignmentFile, FileType } from '@/types/modules/assignments';

const { Title, Text } = Typography;

const fileTypeLabels: Record<FileType, string> = {
  main: 'Main File',
  makefile: 'Makefile',
  memo: 'Memo File',
  spec: 'Specification',
  config: 'Config',
  mark_allocator: 'Mark Allocator',
};

const fileIcons: Record<FileType, React.ReactNode> = {
  main: <FileTextOutlined />,
  makefile: <FileAddOutlined />,
  memo: <FileMarkdownOutlined />,
  spec: <FileZipOutlined />,
  config: <SettingOutlined />,
  mark_allocator: <CodeOutlined />,
};

const AssignmentFiles = () => {
  const { assignment } = useAssignment();
  const module = useModule();
  const [selectedType, setSelectedType] = useState<FileType>('main');
  const [files, setFiles] = useState<AssignmentFile[]>(assignment.files ?? []);

  useEffect(() => {
    setFiles(assignment.files ?? []);
  }, [assignment.files]);

  const handleUpload = async (file: File) => {
    try {
      const res = await uploadAssignmentFile(module.id, assignment.id, selectedType, file);
      message.success(`${fileTypeLabels[selectedType]} "${file.name}" uploaded`);
      setFiles((prev) => [...prev.filter((f) => f.file_type !== selectedType), res.data]);
    } catch (err) {
      message.error('Upload failed');
    }
    return false;
  };

  const handleReplace = (type: FileType) => ({
    beforeUpload: async (file: File) => {
      try {
        const res = await uploadAssignmentFile(module.id, assignment.id, type, file);
        message.success(`${fileTypeLabels[type]} "${file.name}" replaced`);
        setFiles((prev) => prev.map((f) => (f.file_type === type ? res.data : f)));
      } catch (err) {
        message.error('Replacement failed');
      }
      return false;
    },
  });

  const handleDownload = async (id: number) => {
    try {
      await downloadAssignmentFile(module.id, assignment.id, id);
    } catch {
      message.error('Download failed');
    }
  };

  const columns = [
    {
      title: 'File Type',
      dataIndex: 'file_type',
      key: 'file_type',
      render: (type: FileType) => {
        const icon = fileIcons[type] ?? <FileTextOutlined />;
        const label = fileTypeLabels[type] ?? type;
        return (
          <Space>
            {icon}
            {label}
          </Space>
        );
      },
    },
    {
      title: 'Filename',
      dataIndex: 'filename',
      key: 'filename',
      render: (text: string) => (
        <Tooltip title={text}>
          <Text ellipsis style={{ maxWidth: 180, display: 'inline-block' }}>
            {text}
          </Text>
        </Tooltip>
      ),
    },
    {
      title: 'Actions',
      key: 'actions',
      render: (_: any, record: AssignmentFile) => (
        <Space>
          <Button
            size="small"
            icon={<DownloadOutlined />}
            onClick={() => handleDownload(record.id)}
          >
            Download
          </Button>
          <Upload {...handleReplace(record.file_type)} showUploadList={false}>
            <Button size="small" icon={<ReloadOutlined />}>
              Replace
            </Button>
          </Upload>
        </Space>
      ),
    },
  ];

  const segmentedOptions = useMemo(
    () =>
      Object.entries(fileTypeLabels).map(([value, label]) => ({
        value,
        label,
        icon: fileIcons[value as FileType],
      })),
    [],
  );

  return (
    <div className="max-w-5xl space-y-12">
      {/* Upload Section */}
      <div className="border border-gray-200 dark:border-gray-800 rounded-md p-6 space-y-6">
        <Segmented
          block
          value={selectedType}
          onChange={(val) => setSelectedType(val as FileType)}
          options={segmentedOptions}
          className="!mb-2"
        />

        <Upload.Dragger
          accept=".txt,.pdf,.zip,.c,.cpp,.py,.java,.md,.tex"
          showUploadList={false}
          beforeUpload={handleUpload}
          className="rounded border-gray-300 border-dashed p-6"
        >
          <p className="ant-upload-drag-icon">
            <UploadOutlined />
          </p>
          <p className="text-sm text-gray-600">
            Click or drag {fileTypeLabels[selectedType]} to upload
          </p>
        </Upload.Dragger>
      </div>

      {/* Uploaded Files Table */}
      {files.length > 0 && (
        <div className="!mt-12">
          <Title level={4} className="!mb-4">
            Uploaded Files
          </Title>
          <div className="border border-gray-200 dark:border-gray-800 rounded-md overflow-hidden">
            <Table
              columns={columns}
              dataSource={files}
              rowKey="file_type"
              pagination={false}
              size="small"
              className="!border-0"
            />
          </div>
        </div>
      )}
    </div>
  );
};

export default AssignmentFiles;
