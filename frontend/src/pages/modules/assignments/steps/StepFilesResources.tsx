import { useEffect, useState } from 'react';
import { Typography, Upload, message, Button, List, Tooltip, Steps } from 'antd';
import { UploadOutlined, DownloadOutlined, CheckCircleFilled } from '@ant-design/icons';

import { useModule } from '@/context/ModuleContext';
import { useAssignmentSetup } from '@/context/AssignmentSetupContext';
import { uploadAssignmentFile, downloadAssignmentFile } from '@/services/modules/assignments';
import type { AssignmentFile } from '@/types/modules/assignments';

// NEW: makefile task utils
import {
  parseTargetsFromMakefileZip,
  createTasksFromMakefileTargets,
} from '@/utils/makefile_tasks';

const { Step } = Steps;
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
    hint: 'Upload a zip containing a Makefile — tasks will be auto-detected.',
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
  const { assignmentId, assignment, refreshAssignment, readiness } = useAssignmentSetup();

  const [selectedType, setSelectedType] = useState<RequiredFileType>('main');
  const [files, setFiles] = useState<AssignmentFile[]>(assignment?.files ?? []);
  const [uploadError, setUploadError] = useState<RequiredFileType | null>(null);

  // NEW: block UI while we parse + create tasks from makefile zip
  const [creatingFromMakefile, setCreatingFromMakefile] = useState(false);

  useEffect(() => {
    setFiles(assignment?.files ?? []);
  }, [assignment?.files]);

  useEffect(() => {
    if (!readiness) return;

    if (!readiness.main_present) {
      setSelectedType('main');
    } else if (!readiness.memo_present) {
      setSelectedType('memo');
    } else if (!readiness.makefile_present) {
      setSelectedType('makefile');
    } else {
      setSelectedType('main');
    }
  }, [readiness]);

  const handleUpload = async (file: File) => {
    if (!assignmentId) {
      message.error('Assignment ID is missing. Please create the assignment first.');
      return false;
    }
    setUploadError(null);

    try {
      const res = await uploadAssignmentFile(module.id, assignmentId, selectedType, file);
      if (res.success) {
        message.success(`${file.name} uploaded as ${fileTypeLabels[selectedType]}`);
        await refreshAssignment?.();

        // If it's a Makefile zip, best-effort parse & create tasks
        if (selectedType === 'makefile') {
          setCreatingFromMakefile(true);
          try {
            const targets = await parseTargetsFromMakefileZip(file);
            if (targets.length > 0) {
              await createTasksFromMakefileTargets(
                module.id,
                assignmentId,
                targets,
                async () => await refreshAssignment?.(),
              );
              message.success(`Added ${targets.length} task(s) from Makefile`);
            }
            // If no targets or parse fails, do nothing (silent skip as requested)
          } catch {
            // silent per requirement
          } finally {
            setCreatingFromMakefile(false);
          }
        }
      } else {
        setUploadError(selectedType);
        message.error(`Upload failed: ${res.message}`);
      }
    } catch {
      setUploadError(selectedType);
      message.error('Unexpected error during upload.');
    }
    return false; // prevent default upload
  };

  const handleDownload = async (fileId: number) => {
    try {
      await downloadAssignmentFile(module.id, assignmentId!, fileId);
    } catch {
      message.error('Download failed');
    }
  };

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
        <Steps
          current={fileConfigs.findIndex((cfg) => cfg.fileType === selectedType)}
          onChange={(idx) => setSelectedType(fileConfigs[idx].fileType)}
          direction="horizontal"
          size="small"
          status={uploadError ? 'error' : undefined}
        >
          {fileConfigs.map((cfg) => {
            const isUploaded =
              (cfg.fileType === 'main' && readiness?.main_present) ||
              (cfg.fileType === 'memo' && readiness?.memo_present) ||
              (cfg.fileType === 'makefile' && readiness?.makefile_present);

            return (
              <Step
                key={cfg.fileType}
                title={cfg.title}
                icon={isUploaded ? <CheckCircleFilled style={{ color: '#1890ff' }} /> : undefined}
              />
            );
          })}
        </Steps>

        <Upload.Dragger
          multiple={currentConfig.multiple}
          accept={currentConfig.accept}
          maxCount={currentConfig.maxCount}
          beforeUpload={handleUpload}
          showUploadList={false}
          disabled={creatingFromMakefile}
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
            <List
              bordered
              itemLayout="horizontal"
              dataSource={filteredFiles}
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
                    <Text ellipsis style={{ maxWidth: 240 }}>
                      {file.filename}
                    </Text>
                  </Tooltip>
                </List.Item>
              )}
              className="!mt-4"
            />
          </div>
        )}
      </div>
    </div>
  );
};

export default StepFilesResources;
