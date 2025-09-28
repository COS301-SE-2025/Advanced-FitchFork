// src/pages/modules/assignments/Tasks/sections/OverwriteFilesSection.tsx
import React, { useState } from 'react';
import { Button, Typography, Upload, Tag, Tooltip } from 'antd';
import type { UploadFile, RcFile, UploadProps } from 'antd/es/upload/interface';
import SettingsGroup from '@/components/SettingsGroup';
import { message } from '@/utils/message';
import { uploadOverwriteFiles } from '@/services/modules/assignments/overwrite_files/post';
import { downloadOverwriteFile } from '@/services/modules/assignments/overwrite_files/get';
import { deleteOverwriteFiles } from '@/services/modules/assignments/overwrite_files/delete';
import { useTasksPage } from '../context';

const { Dragger } = Upload;

const OverwriteFilesSection: React.FC = () => {
  const { moduleId, assignmentId, selectedTask, setSelectedTask } = useTasksPage();

  const [fileList, setFileList] = useState<UploadFile<RcFile>[]>([]);
  const [uploading, setUploading] = useState(false);
  const [downloading, setDownloading] = useState(false);
  const [deleting, setDeleting] = useState(false);

  if (!selectedTask) return null;
  const hasFile = !!selectedTask.has_overwrite_files;

  // Only allow .zip and block auto-upload
  const beforeUpload = (file: RcFile) => {
    if (!/\.zip$/i.test(file.name)) {
      message.error('Please choose a .zip file.');
      return Upload.LIST_IGNORE;
    }
    return false; // manual upload mode
  };

  const onChange: UploadProps['onChange'] = ({ fileList: newList }) => {
    setFileList(newList as UploadFile<RcFile>[]);
  };

  const onRemove = (file: UploadFile) => {
    setFileList((prev) => prev.filter((f) => f.uid !== file.uid));
    return true;
  };

  const clearSelection = () => setFileList([]);

  // Convert UploadFile[] → FileList for the service
  const toFileList = (ufs: UploadFile<RcFile>[]) => {
    const dt = new DataTransfer();
    for (const f of ufs) {
      const raw = f.originFileObj;
      if (raw instanceof File) dt.items.add(raw);
    }
    return dt.files;
  };

  const onUpload = async () => {
    if (fileList.length === 0) {
      message.warning('Drag in or choose at least one file.');
      return;
    }
    const files = toFileList(fileList);
    if (files.length === 0) {
      message.error('No valid file selected.');
      return;
    }

    try {
      setUploading(true);
      const res = await uploadOverwriteFiles(moduleId, assignmentId, selectedTask.id, files);
      if (res.success) {
        message.success(res.message || 'Uploaded.');
        clearSelection();
        setSelectedTask((prev) => (prev ? { ...prev, has_overwrite_files: true } : prev));
      } else {
        message.error(res.message || 'Upload failed.');
      }
    } catch (e) {
      console.error(e);
      message.error('Upload failed.');
    } finally {
      setUploading(false);
    }
  };

  const onDownload = async () => {
    if (!hasFile) {
      message.info('Nothing to download yet.');
      return;
    }
    try {
      setDownloading(true);
      await downloadOverwriteFile(moduleId, assignmentId, selectedTask.id);
      message.success('Downloaded.');
    } catch (e) {
      console.error(e);
      message.error('Download failed.');
    } finally {
      setDownloading(false);
    }
  };

  const onDeleteAll = async () => {
    if (!hasFile) {
      message.info('Nothing to delete.');
      return;
    }
    try {
      setDeleting(true);
      const res = await deleteOverwriteFiles(moduleId, assignmentId, selectedTask.id);
      if (res.success) {
        message.success(res.message || 'Deleted.');
        setSelectedTask((prev) => (prev ? { ...prev, has_overwrite_files: false } : prev));
      } else {
        message.error(res.message || 'Delete failed.');
      }
    } catch (e) {
      console.error(e);
      message.error('Delete failed.');
    } finally {
      setDeleting(false);
    }
  };

  return (
    <SettingsGroup
      title="Overwrite Files"
      description={
        <>
          Upload an archive whose contents are applied on top of each student’s submission when this
          task runs. Files with the same path/name replace the student’s copy; new files are added.
        </>
      }
    >
      <div>
        {/* Status row */}
        <div className="flex items-center justify-between mb-2">
          <Typography.Text type="secondary">
            {hasFile ? 'An overlay is set and will be used for this task.' : 'No overlay set yet.'}
          </Typography.Text>
          <Tag color={hasFile ? 'green' : 'default'}>{hasFile ? 'Ready' : 'Not set'}</Tag>
        </div>

        <Dragger
          multiple
          accept=".zip,application/zip,application/x-zip-compressed"
          disabled={uploading || downloading || deleting}
          beforeUpload={beforeUpload}
          onChange={onChange}
          onRemove={onRemove}
          fileList={fileList}
          itemRender={(originNode) => originNode}
          className="!p-0"
        >
          <p className="ant-upload-drag-icon" />
          <p className="ant-upload-text">Drag & drop files here</p>
          <p className="ant-upload-hint">…or click to choose</p>
        </Dragger>

        <div className="mt-4">
          <div className="flex flex-col sm:flex-row sm:flex-wrap gap-2">
            <Button
              type="primary"
              onClick={onUpload}
              loading={uploading}
              disabled={fileList.length === 0}
              className="w-full sm:w-auto"
            >
              Upload selected
            </Button>
            <Tooltip title={hasFile ? '' : 'Nothing to download yet'}>
              <Button
                onClick={onDownload}
                loading={downloading}
                disabled={!hasFile}
                className="w-full sm:w-auto"
              >
                Download latest
              </Button>
            </Tooltip>
            <Tooltip title={hasFile ? '' : 'Nothing to delete'}>
              <Button
                danger
                onClick={onDeleteAll}
                loading={deleting}
                disabled={!hasFile}
                className="w-full sm:w-auto"
              >
                Delete all
              </Button>
            </Tooltip>
            <Button
              onClick={clearSelection}
              disabled={fileList.length === 0}
              className="w-full sm:w-auto"
            >
              Clear selection
            </Button>
          </div>
        </div>

        <Typography.Paragraph type="secondary" className="!mb-0 mt-2">
          The newest upload is used. Delete to revert to the student’s files only.
        </Typography.Paragraph>
      </div>
    </SettingsGroup>
  );
};

export default OverwriteFilesSection;
