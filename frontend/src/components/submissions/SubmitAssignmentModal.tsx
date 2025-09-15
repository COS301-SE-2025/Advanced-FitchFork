import { useEffect, useState } from 'react';
import { Modal, Upload, Button, Checkbox, Typography, Space, Tag } from 'antd';
import { CloudUploadOutlined, UploadOutlined, DeleteOutlined } from '@ant-design/icons';
import Tip from '@/components/common/Tip';
import ArchivePreview from '@/components/common/ArchivePreview';

type Props = {
  open: boolean;
  loading?: boolean;
  onClose: () => void;
  onSubmit: (file: File, isPractice: boolean, attestsOwnership: boolean) => Promise<void> | void;

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
  const [attestsOwnership, setAttestsOwnership] = useState<boolean>(false);

  useEffect(() => {
    if (!open) {
      setFile(null);
      setIsPractice(defaultIsPractice);
      setAttestsOwnership(false);
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
    if (!file || loading || !attestsOwnership) return;
    const effectivePractice = allowPractice ? isPractice : false;
    await onSubmit(file, effectivePractice, attestsOwnership);
  };

  const submitDisabled = !file || !attestsOwnership || loading;

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
            <div className="mt-3 space-y-3">
              <div className="flex items-center justify-between rounded-md border border-gray-200 dark:border-gray-700 bg-white dark:bg-gray-950 p-2">
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

              <ArchivePreview file={file} className="rounded-md" />
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
          to="/help/assignments/submissions/how-to-submit"
          newTab
        />

        {/* Ownership attestation (required) – bottom; separate checkbox + text */}
        <div className="flex flex-col gap-1">
          <div className="flex items-start gap-3">
            <Checkbox
              id="ownership-attestation"
              checked={attestsOwnership}
              onChange={(e) => setAttestsOwnership(e.target.checked)}
              disabled={loading}
              data-cy="ownership-attestation"
              aria-required
              aria-describedby="ownership-attestation-text"
              className="mt-0.5"
            />
            <label
              htmlFor="ownership-attestation"
              id="ownership-attestation-text"
              className="text-sm leading-5 text-gray-800 dark:text-gray-200"
              data-cy="ownership-attestation-text"
            >
              I confirm that this submission is <strong>entirely my own work</strong> and that I
              have not shared or received unauthorized assistance. I understand that violations may
              result in disciplinary action.
            </label>
          </div>

          {file && !attestsOwnership && (
            <div className="pl-6">
              <Text type="danger" className="!text-xs">
                You must confirm ownership before submitting.
              </Text>
            </div>
          )}
        </div>

        {/* Practice toggle + actions */}
        <div className="flex flex-col sm:flex-row sm:items-center gap-3">
          {allowPractice && (
            <Checkbox
              checked={isPractice}
              onChange={(e) => setIsPractice(e.target.checked)}
              disabled={loading}
              className="sm:mr-auto"
            >
              This is a practice submission
            </Checkbox>
          )}

          <Space className="sm:ml-auto">
            <Button onClick={onClose} disabled={loading} data-cy="submit-modal-cancel">
              Cancel
            </Button>
            <Button
              type="primary"
              icon={<UploadOutlined />}
              onClick={handleSubmit}
              loading={loading}
              disabled={submitDisabled}
              data-cy="submit-modal-submit"
              aria-disabled={submitDisabled}
              aria-describedby={!attestsOwnership ? 'ownership-attestation-text' : undefined}
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
