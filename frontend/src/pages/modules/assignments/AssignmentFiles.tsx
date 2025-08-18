// pages/modules/assignments/files/AssignmentFiles.tsx
import { useEffect, useMemo, useState } from 'react';
import {
  Typography,
  Button,
  Empty,
  Tooltip,
  Space,
  Card,
  Segmented,
  Alert,
  Upload,
  Divider,
} from 'antd';
import {
  UploadOutlined,
  DownloadOutlined,
  FileTextOutlined,
  FileZipOutlined,
  FileMarkdownOutlined,
  FileProtectOutlined,
  FileUnknownOutlined,
  FileTwoTone,
  ReloadOutlined,
} from '@ant-design/icons';

import { useAssignment } from '@/context/AssignmentContext';
import { useModule } from '@/context/ModuleContext';
import { downloadAssignmentFile, uploadAssignmentFile } from '@/services/modules/assignments';

import type { AssignmentFile, AssignmentFileType } from '@/types/modules/assignments';
import { useViewSlot } from '@/context/ViewSlotContext';
import DateTime from '@/components/common/DateTime';
import { message } from '@/utils/message';

const { Text } = Typography;

// Hide mark_allocator entirely
type VisibleFileType = Exclude<AssignmentFileType, 'mark_allocator'>;

const fileTypeLabels: Record<VisibleFileType, string> = {
  main: 'Main File',
  makefile: 'Makefile',
  memo: 'Memo File',
  spec: 'Specification',
  config: 'Config',
};

// Allowed extensions per type (enforced client-side)
const COMPRESSED = '.zip,.tar,.gz,.tgz,.bz2,.tbz2,.7z';
const ACCEPTS_BY_TYPE: Partial<Record<AssignmentFileType, string>> = {
  main: COMPRESSED,
  memo: COMPRESSED,
  makefile: COMPRESSED,
  config: '.json',
  spec: '.pdf,.md,.txt',
};

function iconForFilename(name?: string) {
  const n = (name ?? '').toLowerCase();
  if (
    n.endsWith('.zip') ||
    n.endsWith('.tar') ||
    n.endsWith('.gz') ||
    n.endsWith('.tgz') ||
    n.endsWith('.bz2') ||
    n.endsWith('.tbz2') ||
    n.endsWith('.7z')
  )
    return <FileZipOutlined />;
  if (n.endsWith('.md')) return <FileMarkdownOutlined />;
  if (n.endsWith('.pdf') || n.endsWith('.txt')) return <FileTextOutlined />;
  if (n.endsWith('.json') || n.endsWith('.yml') || n.endsWith('.yaml') || n.endsWith('.toml'))
    return <FileProtectOutlined />;
  return <FileUnknownOutlined />;
}

// normalize server file_type into our TS union (fallback to current UI selection)
const normalizeType = (t: unknown, fallback: AssignmentFileType): AssignmentFileType => {
  const v = String(t ?? '').toLowerCase();
  const allowed = ['main', 'makefile', 'memo', 'spec', 'config'] as const; // mark_allocator excluded
  return (allowed as readonly string[]).includes(v as any) ? (v as AssignmentFileType) : fallback;
};

// validate file name by extension list
const hasAllowedExtension = (filename: string, acceptList?: string) => {
  if (!acceptList) return true;
  const exts = acceptList.split(',').map((s) => s.trim().toLowerCase());
  const lower = filename.toLowerCase();
  return exts.some((ext) => {
    if (!ext) return false;
    if (ext === 'makefile') return lower.endsWith('makefile');
    if (ext.startsWith('.')) return lower.endsWith(ext);
    return lower.endsWith(ext);
  });
};

const AssignmentFiles = () => {
  const { assignment, refreshAssignment } = useAssignment();
  const module = useModule();
  const { setValue } = useViewSlot();

  // Start on a visible type
  const [selectedType, setSelectedType] = useState<VisibleFileType>('main');
  const [files, setFiles] = useState<AssignmentFile[]>(assignment.files ?? []);
  const [uploading, setUploading] = useState(false);

  useEffect(() => {
    setValue(
      <Typography.Text className="text-base font-medium text-gray-900 dark:text-gray-100 truncate">
        Files
      </Typography.Text>,
    );
  }, [setValue]);

  useEffect(() => {
    setFiles(assignment.files ?? []);
  }, [assignment.files]);

  const filesForSelectedType = useMemo(
    () => files.filter((f) => f.file_type === selectedType),
    [files, selectedType],
  );

  const current = filesForSelectedType[0]; // one-per-type (latest wins)

  const handleUpload = async (file: File) => {
    // Enforce extension policy
    const accept = ACCEPTS_BY_TYPE[selectedType];
    if (!hasAllowedExtension(file.name, accept)) {
      message.error(
        selectedType === 'config'
          ? 'Config must be a .json file.'
          : 'This file type must be a compressed archive (e.g., .zip, .tar, .gz, .7z).',
      );
      return false;
    }

    setUploading(true);
    try {
      const res = await uploadAssignmentFile(module.id, assignment.id, selectedType, file);

      // Support both { data: file } and { data: { file } }
      const serverFile = (res as any)?.data?.file ?? (res as any)?.data ?? null;

      if (!serverFile) {
        message.success(`${fileTypeLabels[selectedType]} “${file.name}” uploaded.`);
        await refreshAssignment();
        return true;
      }

      const normalized: AssignmentFile = {
        ...serverFile,
        file_type: normalizeType(serverFile.file_type, selectedType),
      };

      message.success(
        `${fileTypeLabels[selectedType]} “${file.name}” uploaded and set as current file.`,
      );

      // Optimistic update
      setFiles((prev) => [...prev.filter((f) => f.file_type !== normalized.file_type), normalized]);

      // Authoritative refresh
      await refreshAssignment();

      return true;
    } catch {
      message.error('Upload failed');
      return false;
    } finally {
      setUploading(false);
    }
  };

  const customRequest: NonNullable<React.ComponentProps<typeof Upload>['customRequest']> = async ({
    file,
    onSuccess,
    onError,
  }) => {
    const ok = await handleUpload(file as File);
    if (ok) onSuccess?.(true as any);
    else onError?.(new Error('upload failed'));
  };

  const handleDownload = async (id: number) => {
    try {
      await downloadAssignmentFile(module.id, assignment.id, id);
      message.success('Download started');
    } catch {
      message.error('Download failed');
    }
  };

  const uploaderDisabled = uploading;
  const acceptString = ACCEPTS_BY_TYPE[selectedType] ?? '.pdf,.md,.txt'; // sensible default for spec

  return (
    <div className="max-w-4xl">
      <Card
        className="bg-white dark:bg-gray-900 border border-gray-200 dark:border-gray-800 rounded-xl"
        bodyStyle={{ padding: 24 }}
        title={
          <div className="flex flex-wrap items-center gap-3">
            <FileTwoTone twoToneColor="#1677ff" />
            <span className="font-medium">Assignment Files</span>
            <span className="hidden sm:inline text-gray-400">•</span>
            <Segmented
              size="small"
              value={selectedType}
              onChange={(val) => setSelectedType(val as VisibleFileType)}
              // labels NOT bold
              options={Object.entries(fileTypeLabels).map(([value, label]) => ({
                value,
                label: <span className="font-normal">{label}</span>,
              }))}
            />
          </div>
        }
        extra={
          <Tooltip title="Reload list">
            <Button
              icon={<ReloadOutlined />}
              onClick={async () => {
                await refreshAssignment();
                setFiles(assignment.files ?? []);
              }}
              disabled={uploading}
            />
          </Tooltip>
        }
      >
        <Space direction="vertical" size="large" className="w-full">
          {/* Policy notice per type */}
          {selectedType === 'config' ? (
            <Alert
              type="info"
              showIcon
              message={`${fileTypeLabels[selectedType]} • Upload policy`}
              description="Config must be a single .json file. Uploading a new file replaces the current one."
            />
          ) : selectedType === 'main' || selectedType === 'memo' || selectedType === 'makefile' ? (
            <Alert
              type="info"
              showIcon
              message={`${fileTypeLabels[selectedType]} • Upload policy`}
              description="This file must be uploaded as a compressed archive (e.g., .zip, .tar.gz, .7z). Uploading a new file replaces the current one."
            />
          ) : (
            <Alert
              type="info"
              showIcon
              message={`${fileTypeLabels[selectedType]} • Upload & Replace`}
              description="Uploading a new file will overwrite the existing one for this type."
            />
          )}

          <Upload.Dragger
            key={selectedType} // force remount per type
            customRequest={customRequest}
            accept={acceptString}
            showUploadList={false}
            disabled={uploaderDisabled}
            className="rounded border-gray-300 border-dashed p-6 dark:bg-black/10"
          >
            <p className="ant-upload-drag-icon">
              <UploadOutlined />
            </p>
            <p className="text-sm text-gray-700 dark:text-gray-200">
              {uploaderDisabled
                ? 'Upload in progress…'
                : `Click or drag ${fileTypeLabels[selectedType]} here to upload`}
            </p>
            {!uploaderDisabled && (
              <p className="text-xs text-gray-500 dark:text-gray-400 mt-1">
                This will replace the current file for this type.
              </p>
            )}
          </Upload.Dragger>

          <Divider className="!my-0" />

          {current ? (
            <Card
              size="small"
              className="border border-gray-200 dark:border-gray-800 bg-gray-50 dark:bg-gray-950"
            >
              <div className="flex items-center justify-between gap-3">
                <div className="min-w-0 flex items-center gap-3">
                  {iconForFilename(current.filename)}
                  <Tooltip title={current.filename}>
                    <Text ellipsis className="max-w-[320px] sm:max-w-[520px]">
                      {current.filename}
                    </Text>
                  </Tooltip>
                </div>
                <Space size="small" align="center" className="shrink-0">
                  {'updated_at' in current && (current as any).updated_at ? (
                    <Text type="secondary" className="hidden sm:inline">
                      <DateTime value={(current as any).updated_at} variant="compact" />
                    </Text>
                  ) : null}
                  <Button
                    size="small"
                    icon={<DownloadOutlined />}
                    onClick={() => handleDownload(current.id)}
                  >
                    Download
                  </Button>
                </Space>
              </div>
            </Card>
          ) : (
            <Empty
              image={Empty.PRESENTED_IMAGE_SIMPLE}
              description={
                <div className="text-gray-700 dark:text-gray-300">
                  No file uploaded for {fileTypeLabels[selectedType]}
                </div>
              }
            />
          )}
        </Space>
      </Card>
    </div>
  );
};

export default AssignmentFiles;
