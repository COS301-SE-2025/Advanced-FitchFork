import { useEffect, useMemo, useState, useCallback } from 'react';
import { useParams } from 'react-router-dom';
import { Typography, Upload, Button, Tooltip, Empty, Alert } from 'antd';
import {
  UploadOutlined,
  DownloadOutlined,
  FileZipOutlined,
  FileMarkdownOutlined,
  FileTextOutlined,
  FileProtectOutlined,
  FileUnknownOutlined,
  ThunderboltOutlined,
} from '@ant-design/icons';

import { useAssignment } from '@/context/AssignmentContext';
import { useModule } from '@/context/ModuleContext';
import { useViewSlot } from '@/context/ViewSlotContext';
import {
  downloadAssignmentFile,
  uploadAssignmentFile,
  fetchAssignmentFileBlob,
} from '@/services/modules/assignments';
import { ASSIGNMENT_FILE_TYPES, type AssignmentFileType } from '@/types/modules/assignments';
import DateTime from '@/components/common/DateTime';
import { message } from '@/utils/message';
import {
  parseTargetsFromMakefileZip,
  createTasksFromMakefileTargets,
} from '@/utils/makefile_tasks';
import Tip from '@/components/common/Tip';
import useConfigBackTo from '@/hooks/useConfigBackTo';

const { Text, Paragraph, Title } = Typography;

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

const HELP_LINKS: Record<VisibleFileType, string> = {
  spec: '/help/assignments/files/specification',
  main: '/help/assignments/files/main-files',
  memo: '/help/assignments/files/memo-files',
  makefile: '/help/assignments/files/makefile',
};

const TYPE_SUMMARIES: Record<VisibleFileType, string> = {
  spec: 'Share the official brief and any supporting material students depend on.',
  main: 'Distribute the starter entry point students clone when they begin the assignment.',
  memo: 'Keep the latest marking memo and rubrics in sync for the grading team.',
  makefile: 'Upload the automation bundle that powers your build, test, and task generation.',
};

const TYPE_TIPS: Record<VisibleFileType, string[]> = {
  spec: [
    'Bundle skeleton code alongside the PDF so students download everything at once.',
    'Add a revision note inside the document when you replace the specification.',
  ],
  main: [
    'Keep the entry point lean—store large resources in separate archives if needed.',
    'Document required tooling (compilers, runtime flags) in a README inside the archive.',
  ],
  memo: [
    'Include sample solutions, rubrics, and marking notes in the archive for easy reference.',
    'Re-upload the memo whenever policies change so the checklist reflects the latest state.',
  ],
  makefile: [
    'Use dedicated targets for build, test, and clean to unlock automated workflows.',
    'After uploading, generate tasks directly from the Makefile to seed marking criteria.',
  ],
};

const formatExtension = (value: string) => {
  const lower = value.trim().toLowerCase();

  switch (lower) {
    case '.tgz':
      return 'TAR.GZ';
    case '.tbz2':
      return 'TBZ2';
    case '.tar':
      return 'TAR';
    case '.gz':
      return 'GZ';
    case '.bz2':
      return 'BZ2';
    case '.zip':
      return 'ZIP';
    case '.7z':
      return '7Z';
    default:
      return lower.replace(/^\./, '').toUpperCase();
  }
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
  useConfigBackTo();
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
  const [generatingTasks, setGeneratingTasks] = useState(false);
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

  const handleGenerateTasksFromMakefile = useCallback(async () => {
    if (!current || current.file_type !== 'makefile') return;
    if (generatingTasks) return;

    setGeneratingTasks(true);
    try {
      const blob = await fetchAssignmentFileBlob(module.id, assignment.id, current.id);
      const file = new File([blob], current.filename, {
        type: blob.type || 'application/zip',
      });

      const targets = await parseTargetsFromMakefileZip(file);
      if (!targets.length) {
        message.info('No runnable targets were detected in the Makefile.');
        return;
      }

      const created = await createTasksFromMakefileTargets(
        module.id,
        assignment.id,
        targets,
        refreshAssignment,
      );

      if (created > 0) {
        message.success(`Generated ${created} task${created === 1 ? '' : 's'} from the Makefile.`);
      } else {
        message.info('No new tasks were created from the Makefile.');
      }
    } catch (err) {
      message.error('Failed to generate tasks from the Makefile.');
      // eslint-disable-next-line no-console
      console.error(err);
    } finally {
      setGeneratingTasks(false);
    }
  }, [assignment.id, current, generatingTasks, module.id, refreshAssignment]);

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

  const policyDesc =
    safeType === 'spec'
      ? 'Upload either a single document (.pdf, .md, .txt) OR a compressed archive (.zip, .tar.gz, .7z) that contains your specification (e.g., a PDF) plus any starter skeleton code, examples, or resources. Uploading a new file replaces the current one.'
      : 'Upload a compressed archive (e.g., .zip, .tar.gz, .7z). Uploading a new file replaces the current one.';

  const helpLink = HELP_LINKS[safeType];
  const summaryCopy = TYPE_SUMMARIES[safeType];
  const quickTips = TYPE_TIPS[safeType];

  const allowedBadges = useMemo(() => {
    const raw = accept
      .split(',')
      .map((ext) => ext.trim())
      .filter(Boolean);
    const unique = Array.from(new Set(raw));
    return unique.map(formatExtension);
  }, [accept]);

  return (
    <div className="w-full max-w-4xl space-y-8">
      <div className="space-y-2">
        <div className="flex items-center gap-3">
          <Title level={4} className="!m-0 !text-gray-900 dark:!text-gray-100">
            {label}
          </Title>
          {helpLink ? <Tip iconOnly newTab to={helpLink} text={`Open ${label} help`} /> : null}
        </div>
        <Paragraph className="!m-0 !text-sm !text-gray-600 dark:!text-gray-300">
          {summaryCopy}
        </Paragraph>
      </div>

      <div className="rounded-xl border-gray-200 dark:border-gray-700 bg-white dark:bg-gray-900">
        <Upload.Dragger
          key={safeType}
          customRequest={customRequest}
          accept={accept}
          showUploadList={false}
          disabled={uploading}
          multiple={false}
          className="!border-none !bg-transparent px-6 py-10 text-center"
        >
          <div className="space-y-2">
            <div className="text-3xl text-gray-500 dark:text-gray-300">
              <UploadOutlined />
            </div>
            <Paragraph className="!m-0 !text-sm !text-gray-700 dark:!text-gray-200">
              {uploading
                ? 'Upload in progress…'
                : safeType === 'spec'
                  ? `Click or drag your ${label.toLowerCase()} here — upload a single .pdf/.md/.txt or bundle the brief and skeleton code in an archive.`
                  : `Click or drag the ${label.toLowerCase()} archive here to replace the current version.`}
            </Paragraph>
            {!uploading && (
              <Paragraph className="!m-0 !text-xs !text-gray-500 dark:!text-gray-400">
                Uploading a new file replaces the current one for everyone.
              </Paragraph>
            )}
          </div>
        </Upload.Dragger>
      </div>

      <div className="rounded-xl border border-gray-200 dark:border-gray-800 bg-white dark:bg-gray-900 px-4 py-5 space-y-4">
        <div className="space-y-1">
          <Text className="!text-sm font-medium text-gray-800 dark:text-gray-100">
            Upload guidelines
          </Text>
          <Paragraph className="!m-0 !text-xs !text-gray-600 dark:!text-gray-300 leading-relaxed">
            {policyDesc}
          </Paragraph>
        </div>

        <div className="space-y-2">
          <Text className="!text-sm font-medium text-gray-800 dark:text-gray-100">
            Allowed formats
          </Text>
          <div className="flex flex-wrap gap-2">
            {allowedBadges.map((ext) => (
              <span
                key={ext}
                className="inline-flex items-center rounded-full border border-gray-300 dark:border-gray-700 bg-gray-50 dark:bg-gray-950 px-3 py-1 text-xs font-medium text-gray-600 dark:text-gray-300"
              >
                {ext}
              </span>
            ))}
          </div>
        </div>

        <div className="space-y-2">
          <Text className="!text-sm font-medium text-gray-800 dark:text-gray-100">Quick tips</Text>
          <ul className="list-disc space-y-2 pl-5 text-xs text-gray-600 dark:text-gray-300">
            {quickTips.map((tip) => (
              <li key={tip}>{tip}</li>
            ))}
          </ul>
        </div>
      </div>

      {current ? (
        <div className="rounded-xl border border-gray-200 dark:border-gray-800 bg-white dark:bg-gray-900 px-4 py-4">
          <div className="flex flex-col gap-4 sm:flex-row sm:items-center sm:justify-between">
            <div className="flex min-w-0 items-center gap-3">
              <span className="text-lg text-gray-500 dark:text-gray-300">
                {iconForFilename(current.filename)}
              </span>
              <Tooltip title={current.filename}>
                <Text
                  ellipsis
                  className="!m-0 text-sm font-medium text-gray-900 dark:text-gray-100"
                >
                  {current.filename}
                </Text>
              </Tooltip>
            </div>
            <div className="flex flex-wrap items-center gap-3">
              {'updated_at' in current && (current as any).updated_at ? (
                <span className="text-xs text-gray-500 dark:text-gray-400">
                  <DateTime value={(current as any).updated_at} variant="compact" />
                </span>
              ) : null}
              <Button size="small" icon={<DownloadOutlined />} onClick={download}>
                Download
              </Button>
            </div>
          </div>
        </div>
      ) : (
        <div className="rounded-xl border border-dashed border-gray-200 dark:border-gray-800 bg-white dark:bg-gray-900 px-4 py-10">
          <Empty
            image={Empty.PRESENTED_IMAGE_SIMPLE}
            description={
              <span className="text-sm text-gray-600 dark:text-gray-400">No file uploaded yet</span>
            }
          />
        </div>
      )}

      {safeType === 'makefile' && current && (
        <div className="rounded-xl border border-gray-200 dark:border-gray-800 bg-white dark:bg-gray-900 px-4 py-4 space-y-2">
          <div className="flex flex-col gap-2 sm:flex-row sm:items-center sm:justify-between">
            <div>
              <Text className="!text-sm font-medium text-gray-800 dark:text-gray-100">
                Generate tasks from the Makefile
              </Text>
              <Paragraph className="!m-0 !text-xs !text-gray-600 dark:!text-gray-300">
                We will inspect runnable targets and add matching tasks automatically.
              </Paragraph>
            </div>
            <Button
              icon={<ThunderboltOutlined />}
              type="default"
              onClick={() => void handleGenerateTasksFromMakefile()}
              loading={generatingTasks}
              disabled={generatingTasks}
            >
              Generate tasks
            </Button>
          </div>
        </div>
      )}
    </div>
  );
}
