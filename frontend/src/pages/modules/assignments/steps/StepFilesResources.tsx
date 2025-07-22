import { useEffect, useMemo, useState } from 'react';
import { Typography, Segmented, Upload, message, Table, Space, Button } from 'antd';
import { UploadOutlined, DownloadOutlined, CheckCircleFilled } from '@ant-design/icons';

import { useModule } from '@/context/ModuleContext';
import { useAssignmentSetup } from '@/context/AssignmentSetupContext';
import { uploadAssignmentFile, downloadAssignmentFile } from '@/services/modules/assignments';
import type { AssignmentFile } from '@/types/modules/assignments';

const { Title, Paragraph, Text } = Typography;

type RequiredFileType = 'main' | 'memo' | 'makefile';

const fileConfigs: {
  title: string;
  hint: string;
  accept: string;
  fileType: RequiredFileType;
  maxCount?: number;
  multiple?: boolean;
}[] = [
  {
    title: 'Main Files',
    hint: 'Starter files provided to all students (.zip)',
    accept: '.zip',
    fileType: 'main',
    multiple: true,
  },
  {
    title: 'Memo File',
    hint: 'The official memo or marking guide (.zip)',
    accept: '.zip',
    fileType: 'memo',
    maxCount: 1,
  },
  {
    title: 'Makefile',
    hint: 'Required for compiled languages like C/C++ (.zip)',
    accept: '.zip',
    fileType: 'makefile',
    maxCount: 1,
  },
];

const fileTypeLabels: Record<RequiredFileType, string> = {
  main: 'Main File',
  memo: 'Memo File',
  makefile: 'Makefile',
};

const StepFilesResources = () => {
  const module = useModule();
  const { assignmentId, assignment, refreshAssignment, readiness, onStepComplete } =
    useAssignmentSetup();

  const [selectedType, setSelectedType] = useState<RequiredFileType>('main');
  const [files, setFiles] = useState<AssignmentFile[]>(assignment?.files ?? []);

  useEffect(() => {
    setFiles(assignment?.files ?? []);
  }, [assignment?.files]);

  const handleUpload = async (file: File) => {
    if (!assignmentId) {
      message.error('Assignment ID is missing. Please create the assignment first.');
      return false;
    }
    try {
      const res = await uploadAssignmentFile(module.id, assignmentId, selectedType, file);
      if (res.success) {
        message.success(`${file.name} uploaded as ${fileTypeLabels[selectedType]}`);
        await refreshAssignment?.();
        onStepComplete?.();
      } else {
        message.error(`Upload failed: ${res.message}`);
      }
    } catch {
      message.error('Unexpected error during upload.');
    }
    return false;
  };

  const handleDownload = async (fileId: number) => {
    try {
      await downloadAssignmentFile(module.id, assignmentId!, fileId);
    } catch {
      message.error('Download failed');
    }
  };

  const columns = [
    {
      title: 'Filename',
      dataIndex: 'filename',
      key: 'filename',
      render: (name: string) => <Text>{name}</Text>,
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
        </Space>
      ),
    },
  ];

  const segmentedOptions = useMemo(
    () =>
      fileConfigs.map((cfg) => {
        const isUploaded =
          (cfg.fileType === 'main' && readiness?.main_present) ||
          (cfg.fileType === 'memo' && readiness?.memo_present) ||
          (cfg.fileType === 'makefile' && readiness?.makefile_present);

        return {
          value: cfg.fileType,
          label: (
            <span>
              {cfg.title}{' '}
              {isUploaded && <CheckCircleFilled style={{ color: '#1890ff', fontSize: 14 }} />}
            </span>
          ),
        };
      }),
    [readiness],
  );

  const currentConfig = fileConfigs.find((cfg) => cfg.fileType === selectedType)!;
  const filteredFiles = files.filter((f) => f.file_type === selectedType);

  return (
    <div className="space-y-6">
      <div>
        <Title level={3} className="!mb-1">
          Upload Required Assignment Files
        </Title>
        <Paragraph type="secondary" className="!mb-0">
          Upload starter code, memo, or makefile required by students and the marking system.
        </Paragraph>
      </div>

      <div className="!space-y-4">
        <Segmented
          block
          value={selectedType}
          onChange={(val) => setSelectedType(val as RequiredFileType)}
          options={segmentedOptions}
        />

        <Upload.Dragger
          multiple={currentConfig.multiple}
          accept={currentConfig.accept}
          maxCount={currentConfig.maxCount}
          beforeUpload={handleUpload}
          showUploadList={false}
        >
          <p className="ant-upload-drag-icon">
            <UploadOutlined />
          </p>
          <p className="ant-upload-text text-sm">
            Click or drag {currentConfig.title} here to upload
          </p>
          <p className="ant-upload-hint text-xs text-gray-500">{currentConfig.hint}</p>
        </Upload.Dragger>

        {filteredFiles.length > 0 && (
          <div>
            <Title level={5} className="!mt-6 !mb-2">
              Uploaded Files ({currentConfig.title})
            </Title>
            <Table
              columns={columns}
              dataSource={filteredFiles}
              rowKey="id"
              pagination={false}
              size="small"
            />
          </div>
        )}
      </div>
    </div>
  );
};

export default StepFilesResources;
