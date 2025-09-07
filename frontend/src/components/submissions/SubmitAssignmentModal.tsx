import { useEffect, useState } from 'react';
import { Modal, Upload, Button, Checkbox, Typography, Space, Tag } from 'antd';
import { CloudUploadOutlined, UploadOutlined, DeleteOutlined } from '@ant-design/icons';
import Tip from '@/components/common/Tip';

type Props = {
  open: boolean;
  loading?: boolean;
  onClose: () => void;
  onSubmit: (file: File, isPractice: boolean) => Promise<void> | void;

  /** Optional UX props */
  title?: string;
  accept?: string; // e.g. ".zip,.tar,.gz,.tgz"
  maxSizeMB?: number; // e.g. 50
  defaultIsPractice?: boolean;

  /** Show/hide practice option */
  allowPractice?: boolean;
};

const { Text, Title } = Typography;

const SubmitAssignmentModal = ({
  open,
  loading = false,
  onClose,
  onSubmit,
  title = 'Submit Assignment',
  accept = '.zip,.tar,.gz,.tgz',
  maxSizeMB = 50,
  defaultIsPractice = false,
  allowPractice = true,
}: Props) => {
  const [file, setFile] = useState<File | null>(null);
  const [isPractice, setIsPractice] = useState<boolean>(defaultIsPractice);

  useEffect(() => {
    if (!open) {
      setFile(null);
      setIsPractice(defaultIsPractice);
    }
  }, [open, defaultIsPractice]);

  useEffect(() => {
    if (!allowPractice && isPractice) {
      setIsPractice(false);
    }
  }, [allowPractice, isPractice]);

  const beforeUpload: NonNullable<React.ComponentProps<typeof Upload>['beforeUpload']> = (f) => {
    if (maxSizeMB && f.size > maxSizeMB * 1024 * 1024) {
      return Upload.LIST_IGNORE;
    }
    setFile(f);
    return false;
  };

  const handleClear = () => setFile(null);

  const handleSubmit = async () => {
    if (!file || loading) return;
    const effectivePractice = allowPractice ? isPractice : false;
    await onSubmit(file, effectivePractice);
  };

  return (
    <Modal
      open={open}
      onCancel={onClose}
      footer={null}
      centered
      title={
        <Title level={4} className="!mb-0">
          {title}
        </Title>
      }
      destroyOnHidden
      width={600}
    >
      <div className="space-y-4">
        {/* File Upload */}
        <div className="!mt-3">
          <Upload.Dragger
            multiple={false}
            maxCount={1}
            beforeUpload={beforeUpload}
            showUploadList={false}
            accept={accept}
            disabled={loading}
          >
            <p className="ant-upload-drag-icon">
              <CloudUploadOutlined />
            </p>
            <p className="ant-upload-text">Drag & drop your archive here</p>
            <p className="ant-upload-hint">or click to browse</p>
          </Upload.Dragger>

          {file && (
            <div className="mt-3 flex items-center justify-between rounded-md border border-gray-200 dark:border-gray-700 bg-white dark:bg-gray-950 p-2">
              <Space size="small" direction="vertical">
                <Text strong className="!text-gray-900 dark:!text-gray-100">
                  {file.name}
                </Text>
                <Text type="secondary" className="!text-xs">
                  {(file.size / (1024 * 1024)).toFixed(2)} MB
                </Text>
              </Space>
              <Button
                size="small"
                type="text"
                danger
                icon={<DeleteOutlined />}
                onClick={handleClear}
                aria-label="Remove selected file"
              />
            </div>
          )}
        </div>

        {/* Info Boxes */}
        <div className="grid gap-3 sm:grid-cols-3">
          <div className="rounded-lg border border-gray-200 dark:border-gray-800 bg-white dark:bg-gray-950 p-3">
            <div className="text-xs font-medium text-gray-500 dark:text-gray-400">Accepted</div>
            <div className="text-sm text-gray-800 dark:text-gray-200">{accept}</div>
          </div>

          <div className="rounded-lg border border-gray-200 dark:border-gray-800 bg-white dark:bg-gray-950 p-3">
            <div className="text-xs font-medium text-gray-500 dark:text-gray-400">Max size</div>
            <div className="text-sm text-gray-800 dark:text-gray-200">{maxSizeMB} MB</div>
          </div>

          {allowPractice && (
            <div className="rounded-lg border border-gray-200 dark:border-gray-800 bg-white dark:bg-gray-950 p-3">
              <div className="text-xs font-medium text-gray-500 dark:text-gray-400">
                Is Practice
              </div>
              <div className="text-sm">
                <Tag color={isPractice ? 'green' : 'red'}>{isPractice ? 'Yes' : 'No'}</Tag>
              </div>
            </div>
          )}
        </div>

        <Tip
          type="info"
          showIcon
          text="Tip: Make sure your archive contains the correct structure."
          to="/help/submissions"
        />

        {/* Practice toggle + actions */}
        <div className="flex flex-col sm:flex-row sm:items-center gap-3">
          {allowPractice && (
            <Checkbox
              checked={isPractice}
              onChange={(e) => setIsPractice(e.target.checked)}
              disabled={loading}
              className="sm:mr-auto" // pushes actions to the right when checkbox is visible
            >
              This is a practice submission
            </Checkbox>
          )}

          {/* Actions always pinned to the right on wide screens */}
          <Space className="sm:ml-auto">
            <Button onClick={onClose} disabled={loading} data-cy="submit-modal-cancel">
              Cancel
            </Button>
            <Button
              type="primary"
              icon={<UploadOutlined />}
              onClick={handleSubmit}
              loading={loading}
              disabled={!file || loading}
              data-cy="submit-modal-submit"
            >
              Submit
            </Button>
          </Space>
        </div>
      </div>
    </Modal>
  );
};

export default SubmitAssignmentModal;
