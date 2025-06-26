// src/pages/modules/assignments/steps/AssignmentStepUpload.tsx
import { Upload, Typography, message } from 'antd';
import { UploadOutlined } from '@ant-design/icons';
import { useAssignment } from '@/context/AssignmentContext';
import { useModule } from '@/context/ModuleContext';
import { uploadAssignmentFile } from '@/services/modules/assignments';
import type { FileType } from '@/types/modules/assignments';

const { Title, Paragraph, Text } = Typography;

const friendlyLabels: Record<FileType, string> = {
  config: 'Configuration File',
  main: 'Main Source File',
  memo: 'Memo File',
  makefile: 'Makefile',
  spec: 'Specification',
  mark_allocator: 'Mark Allocator',
};

const AssignmentStepUpload = ({ fileType }: { fileType: FileType }) => {
  const { assignment, refreshReadiness } = useAssignment();
  const module = useModule();

  const handleUpload = async (file: File) => {
    try {
      const res = await uploadAssignmentFile(module.id, assignment.id, fileType, file);
      message.success(`Uploaded ${file.name}`);
      refreshReadiness?.();
    } catch (err) {
      message.error('Upload failed');
    }
    return false;
  };

  return (
    <div className="max-w-2xl px-6 py-8 border border-gray-300 dark:border-gray-700 rounded-lg bg-white dark:bg-black/10">
      <Title level={3} className="!mb-2">
        {friendlyLabels[fileType]}
      </Title>
      <Paragraph className="text-gray-600 dark:text-gray-300">
        Please upload the required <Text strong>{friendlyLabels[fileType]}</Text> to continue. This
        step is mandatory before proceeding to the next phase of the assignment setup.
      </Paragraph>

      <Upload.Dragger
        accept=".txt,.pdf,.zip,.c,.cpp,.py,.java,.md,.tex"
        showUploadList={false}
        beforeUpload={handleUpload}
        className="mt-6 border-gray-300 dark:border-gray-600 rounded-md p-6 bg-gray-50 dark:bg-gray-900"
      >
        <p className="ant-upload-drag-icon">
          <UploadOutlined style={{ fontSize: 36 }} />
        </p>
        <p className="text-base text-gray-700 dark:text-gray-200 font-medium">
          Click or drag a file to upload
        </p>
        <p className="text-sm text-gray-500 dark:text-gray-400">
          Accepted formats: .txt, .pdf, .zip, .c, .cpp, .py, .java, .md, .tex
        </p>
      </Upload.Dragger>
    </div>
  );
};

export default AssignmentStepUpload;
