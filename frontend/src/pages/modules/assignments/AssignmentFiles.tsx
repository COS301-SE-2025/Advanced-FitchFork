import { useState } from 'react';
import { Segmented, Upload, Typography, message, Table, Button, Space, Tooltip } from 'antd';
import {
  UploadOutlined,
  DownloadOutlined,
  ReloadOutlined,
  FileTextOutlined,
  FileAddOutlined,
  FileMarkdownOutlined,
  FileZipOutlined,
} from '@ant-design/icons';

const { Title, Text } = Typography;

type FileType = 'Main File' | 'Makefile' | 'Memo File' | 'Specification';

interface UploadedFile {
  name: string;
  type: FileType;
}

const fileIcons: Record<FileType, React.ReactNode> = {
  'Main File': <FileTextOutlined />,
  Makefile: <FileAddOutlined />,
  'Memo File': <FileMarkdownOutlined />,
  Specification: <FileZipOutlined />,
};

const AssignmentFiles = () => {
  const [selectedType, setSelectedType] = useState<FileType>('Main File');
  const [uploadedFiles, setUploadedFiles] = useState<UploadedFile[]>([]);

  const handleUpload = (file: File) => {
    message.success(`${selectedType} "${file.name}" uploaded`);
    setUploadedFiles((prev) => {
      const filtered = prev.filter((f) => f.type !== selectedType);
      return [...filtered, { name: file.name, type: selectedType }];
    });
    return false;
  };

  const handleReplace = (type: FileType) => ({
    beforeUpload: (file: File) => {
      message.success(`${type} "${file.name}" re-uploaded`);
      setUploadedFiles((prev) =>
        prev.map((f) => (f.type === type ? { name: file.name, type } : f)),
      );
      return false;
    },
  });

  const columns = [
    {
      title: 'File Type',
      dataIndex: 'type',
      key: 'type',
      render: (type: FileType) => (
        <Space>
          {fileIcons[type]}
          {type}
        </Space>
      ),
    },
    {
      title: 'Filename',
      dataIndex: 'name',
      key: 'name',
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
      render: (_: any, record: UploadedFile) => (
        <Space>
          <Button
            size="small"
            icon={<DownloadOutlined />}
            onClick={() => message.info(`Downloading ${record.name}`)}
          >
            Download
          </Button>
          <Upload {...handleReplace(record.type)} showUploadList={false}>
            <Button size="small" icon={<ReloadOutlined />}>
              Replace
            </Button>
          </Upload>
        </Space>
      ),
    },
  ];

  return (
    <div className="max-w-5xl space-y-12">
      {/* Upload Section */}
      <div className="border border-gray-200 dark:border-gray-800 rounded-md p-6 space-y-6">
        <Segmented
          block
          value={selectedType}
          onChange={(value) => setSelectedType(value as FileType)}
          options={[
            { label: 'Main File', value: 'Main File', icon: fileIcons['Main File'] },
            { label: 'Makefile', value: 'Makefile', icon: fileIcons['Makefile'] },
            { label: 'Memo File', value: 'Memo File', icon: fileIcons['Memo File'] },
            { label: 'Specification', value: 'Specification', icon: fileIcons['Specification'] },
          ]}
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
          <p className="text-sm text-gray-600">Click or drag {selectedType} to upload</p>
        </Upload.Dragger>
      </div>

      {/* Uploaded Files Table */}
      {uploadedFiles.length > 0 && (
        <div className="!mt-12">
          <Title level={4} className="!mb-4">
            Uploaded Files
          </Title>
          <div className="border border-gray-200 dark:border-gray-800 rounded-md overflow-hidden">
            <Table
              columns={columns}
              dataSource={uploadedFiles}
              rowKey="type"
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
