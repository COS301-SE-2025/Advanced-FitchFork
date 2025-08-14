import { useEffect, useState } from 'react';
import { Select, Upload, Typography, message, Button, Empty, List, Tooltip } from 'antd';
import { UploadOutlined, DownloadOutlined } from '@ant-design/icons';

import { useAssignment } from '@/context/AssignmentContext';
import { useModule } from '@/context/ModuleContext';
import { downloadAssignmentFile, uploadAssignmentFile } from '@/services/modules/assignments';

import type { AssignmentFile, AssignmentFileType } from '@/types/modules/assignments';
import { useViewSlot } from '@/context/ViewSlotContext';

const { Title, Text } = Typography;

const fileTypeLabels: Record<AssignmentFileType, string> = {
  main: 'Main File',
  makefile: 'Makefile',
  memo: 'Memo File',
  spec: 'Specification',
  config: 'Config',
  mark_allocator: 'Mark Allocator',
};

const AssignmentFiles = () => {
  const { assignment } = useAssignment();
  const module = useModule();
  const { setValue } = useViewSlot();

  const [selectedType, setSelectedType] = useState<AssignmentFileType>('main');
  const [files, setFiles] = useState<AssignmentFile[]>(assignment.files ?? []);

  useEffect(() => {
    setValue(
      <Typography.Text
        className="text-base font-medium text-gray-900 dark:text-gray-100 truncate"
        title={'Files'}
      >
        Files
      </Typography.Text>,
    );
  }, []);

  useEffect(() => {
    setFiles(assignment.files ?? []);
  }, [assignment.files]);

  const handleUpload = async (file: File) => {
    try {
      const res = await uploadAssignmentFile(module.id, assignment.id, selectedType, file);
      message.success(`${fileTypeLabels[selectedType]} "${file.name}" uploaded`);
      setFiles((prev) => [...prev.filter((f) => f.file_type !== selectedType), res.data]);
    } catch {
      message.error('Upload failed');
    }
    return false;
  };

  const handleDownload = async (id: number) => {
    try {
      await downloadAssignmentFile(module.id, assignment.id, id);
    } catch {
      message.error('Download failed');
    }
  };

  const filesForSelectedType = files.filter((f) => f.file_type === selectedType);

  return (
    <div className="bg-white dark:bg-gray-900 border border-gray-200 dark:border-gray-700 rounded-md p-6 space-y-6 max-w-3xl">
      <div className="flex flex-wrap gap-2 items-center">
        <Title level={4} className="!m-0">
          Manage
        </Title>
        <Select
          value={selectedType}
          onChange={(val) => setSelectedType(val as AssignmentFileType)}
          size="middle"
          style={{ width: 'auto' }}
          popupMatchSelectWidth={false}
          options={Object.entries(fileTypeLabels).map(([value, label]) => ({
            value,
            label,
          }))}
        />
        <Title level={4} className="!m-0">
          File
        </Title>
      </div>

      <div className="!space-y-6">
        <div>
          <Upload.Dragger
            accept=".txt,.pdf,.zip,.c,.cpp,.py,.java,.md,.tex"
            showUploadList={false}
            beforeUpload={handleUpload}
            className="rounded border-gray-300 border-dashed p-6 dark:bg-black/10"
          >
            <p className="ant-upload-drag-icon">
              <UploadOutlined />
            </p>
            <p className="text-sm text-gray-600 dark:text-gray-300">
              Click or drag {fileTypeLabels[selectedType]} here to upload
            </p>
            <p className="text-xs text-gray-500 dark:text-gray-400 mt-1">
              Uploading a new file will overwrite the existing one (if any).
            </p>
          </Upload.Dragger>
        </div>

        <div>
          {filesForSelectedType.length === 0 ? (
            <Empty
              description={
                <div className="text-gray-700 dark:text-gray-300">
                  No file uploaded for {fileTypeLabels[selectedType]}
                </div>
              }
            />
          ) : (
            <List
              bordered
              itemLayout="horizontal"
              dataSource={filesForSelectedType}
              renderItem={(file) => (
                <List.Item
                  actions={[
                    <Button
                      key="download"
                      size="small"
                      icon={<DownloadOutlined />}
                      onClick={() => handleDownload(file.id)}
                    >
                      Download
                    </Button>,
                  ]}
                >
                  <Tooltip title={file.filename}>
                    <Text ellipsis style={{ maxWidth: 200 }}>
                      {file.filename}
                    </Text>
                  </Tooltip>
                </List.Item>
              )}
            />
          )}
        </div>
      </div>
    </div>
  );
};

export default AssignmentFiles;
