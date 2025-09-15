import { useEffect, useMemo, useState, useCallback } from 'react';
import { useParams } from 'react-router-dom';
import { Typography, Card, Space, Upload, Button, Tooltip, Empty, Alert, Row, Col } from 'antd';
import {
  UploadOutlined,
  DownloadOutlined,
  FileZipOutlined,
  FileMarkdownOutlined,
  FileTextOutlined,
  FileProtectOutlined,
  FileUnknownOutlined,
} from '@ant-design/icons';

import { useAssignment } from '@/context/AssignmentContext';
import { useModule } from '@/context/ModuleContext';
import { useViewSlot } from '@/context/ViewSlotContext';
import { downloadAssignmentFile, uploadAssignmentFile } from '@/services/modules/assignments';
import { ASSIGNMENT_FILE_TYPES, type AssignmentFileType } from '@/types/modules/assignments';
import DateTime from '@/components/common/DateTime';
import { message } from '@/utils/message';

const { Text } = Typography;

/** Visible types = all assignment file types EXCEPT 'mark_allocator' and 'config' */
type VisibleFileType = Exclude<AssignmentFileType, 'mark_allocator' | 'config'>;
const VISIBLE_TYPES = ASSIGNMENT_FILE_TYPES.filter(
  (t): t is VisibleFileType => t !== 'mark_allocator' && t !== 'config',
);

const LABELS: Record<VisibleFileType, string> = {
  spec: 'Specification',
  main: 'Main File',
  memo: 'Memo File',
  makefile: 'Makefile',
};

// Allowed extensions (client-side)
const COMPRESSED = '.zip,.tar,.gz,.tgz,.bz2,.tbz2,.7z';
const ACCEPTS_BY_TYPE: Record<VisibleFileType, string> = {
  // Allow a single doc OR an archive (e.g., include PDF + skeleton code)
  spec: `${COMPRESSED},.pdf,.md,.txt`,
  main: COMPRESSED,
  memo: COMPRESSED,
  makefile: COMPRESSED,
};

function iconForFilename(name?: string) {
  const n = (name ?? '').toLowerCase();
  if (/\.(zip|tar|gz|tgz|bz2|tbz2|7z)$/i.test(n)) return <FileZipOutlined />;
  if (n.endsWith('.md')) return <FileMarkdownOutlined />;
  if (/\.(pdf|txt)$/i.test(n)) return <FileTextOutlined />;
  if (/\.(json|yml|yaml|toml)$/i.test(n)) return <FileProtectOutlined />;
  return <FileUnknownOutlined />;
}

function hasAllowedExtension(filename: string, acceptList: string) {
  if (!acceptList) return true;
  const exts = acceptList.split(',').map((s) => s.trim().toLowerCase());
  const lower = filename.toLowerCase();
  return exts.some((ext) => {
    if (!ext) return false;
    if (ext === 'makefile') return lower.endsWith('makefile');
    if (ext.startsWith('.')) return lower.endsWith(ext);
    return lower.endsWith(ext);
  });
}

export default function AssignmentFilePage() {
  const { fileType } = useParams<{ fileType: string }>();
  const type = (fileType ?? '').toLowerCase() as AssignmentFileType;

  const isVisibleType = (VISIBLE_TYPES as readonly string[]).includes(type as VisibleFileType);
  const safeType: VisibleFileType = isVisibleType ? (type as VisibleFileType) : 'main';
  const label = LABELS[safeType];

  const { assignment, assignmentFiles, refreshAssignment } = useAssignment();
  const module = useModule();
  const { setValue } = useViewSlot();

  useEffect(() => {
    setValue(
      <Text className="text-base font-medium text-gray-900 dark:text-gray-100 truncate">
        {label}
      </Text>,
    );
  }, [label, setValue]);

  // Derive the current file for this type (latest wins)
  const files = assignmentFiles ?? [];
  const current = useMemo(() => files.find((f) => f.file_type === safeType), [files, safeType]);

  const [uploading, setUploading] = useState(false);
  const accept = ACCEPTS_BY_TYPE[safeType];

  const handleUpload = useCallback(
    async (file: File) => {
      if (!hasAllowedExtension(file.name, accept)) {
        message.error(
          safeType === 'spec'
            ? 'Specification must be a .pdf/.md/.txt or a compressed archive (.zip, .tar.gz, .7z) containing the spec (e.g., PDF) and optional skeleton code.'
            : 'This type must be uploaded as a compressed archive (e.g., .zip, .tar.gz, .7z).',
        );
        return false;
      }

      setUploading(true);
      try {
        await uploadAssignmentFile(module.id, assignment.id, safeType as AssignmentFileType, file);
        message.success(`${label} “${file.name}” uploaded.`);
        await refreshAssignment();
        return true;
      } catch {
        message.error('Upload failed');
        return false;
      } finally {
        setUploading(false);
      }
    },
    [accept, assignment.id, label, module.id, refreshAssignment, safeType],
  );

  const customRequest: NonNullable<React.ComponentProps<typeof Upload>['customRequest']> = async ({
    file,
    onSuccess,
    onError,
  }) => {
    const ok = await handleUpload(file as File);
    if (ok) onSuccess?.(true as any);
    else onError?.(new Error('upload failed'));
  };

  const download = async () => {
    if (!current) return;
    try {
      await downloadAssignmentFile(module.id, assignment.id, current.id);
      message.success('Download started');
    } catch {
      message.error('Download failed');
    }
  };

  if (!isVisibleType) {
    return (
      <Alert
        type="warning"
        showIcon
        message="Unsupported or hidden file type"
        description={`The path segment “${fileType}” is not available here. Allowed: ${VISIBLE_TYPES.join(', ')}.`}
      />
    );
  }

  const policyTitle =
    safeType === 'spec' ? 'Upload Policy • Specification' : `Upload Policy • ${label}`;
  const policyDesc =
    safeType === 'spec'
      ? 'Upload either a single document (.pdf, .md, .txt) OR a compressed archive (.zip, .tar.gz, .7z) that contains your specification (e.g., a PDF) plus any starter skeleton code, examples, or resources. Uploading a new file replaces the current one.'
      : 'Upload a compressed archive (e.g., .zip, .tar.gz, .7z). Uploading a new file replaces the current one.';

  return (
    <div className="w-full max-w-6xl overflow-x-hidden">
      <Row
        gutter={[{ xs: 0, sm: 16, md: 16 }, 16]}
        wrap
        style={{ marginInline: 0 }}
        align="stretch"
      >
        {/* Alert RIGHT on desktop, ABOVE on mobile */}
        <Col xs={{ span: 24, order: 1 }} md={{ span: 8, order: 2 }}>
          <Alert type="info" showIcon message={policyTitle} description={policyDesc} />
        </Col>

        {/* Card LEFT on desktop, BELOW on mobile */}
        <Col xs={{ span: 24, order: 2 }} md={{ span: 16, order: 1 }}>
          <Card
            className="bg-white dark:bg-gray-900 border border-gray-200 dark:border-gray-800 rounded-xl h-full"
            title={<span className="font-medium">{label}</span>}
            bodyStyle={{ padding: 24 }}
          >
            <Space direction="vertical" size="large" className="w-full">
              {/* Dragger */}
              <Upload.Dragger
                key={safeType}
                customRequest={customRequest}
                accept={accept}
                showUploadList={false}
                disabled={uploading}
                className="w-full rounded border-gray-300 border-dashed p-6 dark:bg-black/10"
              >
                <p className="ant-upload-drag-icon">
                  <UploadOutlined />
                </p>
                <p className="text-sm text-gray-700 dark:text-gray-200">
                  {uploading
                    ? 'Upload in progress…'
                    : safeType === 'spec'
                      ? `Click or drag your ${label} here — either a single .pdf/.md/.txt, or a .zip/.tar.gz/.7z containing the PDF and any skeleton code.`
                      : `Click or drag ${label} here to upload`}
                </p>
                {!uploading && (
                  <p className="text-xs text-gray-500 dark:text-gray-400 mt-1">
                    This will replace the current {label.toLowerCase()}.
                  </p>
                )}
              </Upload.Dragger>

              {/* Current file */}
              {current ? (
                <Card
                  size="small"
                  className="border border-gray-200 dark:border-gray-800 bg-gray-50 dark:bg-gray-950"
                >
                  <div className="flex items-center justify-between gap-3">
                    <div className="min-w-0 flex-1 flex items-center gap-3">
                      {iconForFilename(current.filename)}
                      <Tooltip title={current.filename}>
                        <Text ellipsis className="block w-full">
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
                      <Button size="small" icon={<DownloadOutlined />} onClick={download}>
                        Download
                      </Button>
                    </Space>
                  </div>
                </Card>
              ) : (
                <Empty
                  image={Empty.PRESENTED_IMAGE_SIMPLE}
                  description={
                    <div className="text-gray-700 dark:text-gray-300">No file uploaded</div>
                  }
                />
              )}
            </Space>
          </Card>
        </Col>
      </Row>
    </div>
  );
}
