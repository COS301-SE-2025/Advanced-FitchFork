import { useState, useMemo } from 'react';
import { Modal, Space, Typography, Radio, Select, Tooltip, Input, Alert, Divider } from 'antd';
import { InfoCircleOutlined } from '@ant-design/icons';
import { runMossCheck } from '@/services/modules/assignments/plagiarism';
import { MOSS_FILTER_MODES, type MossFilterMode } from '@/types/modules/assignments/plagiarism';
import { message } from '@/utils/message';

type Props = {
  open: boolean;
  onClose: () => void;
  moduleId: number;
  assignmentId: number;
  onRan?: () => void;
  // kept for compatibility, but no longer shown in the UI:
  latestReportUrl?: string | null;
  latestGeneratedAt?: string | null;
  hasArchive?: boolean;
  latestArchiveAt?: string | null;
};

const MossRunModal: React.FC<Props> = ({ open, onClose, moduleId, assignmentId, onRan }) => {
  const [filterMode, setFilterMode] = useState<MossFilterMode>('all');
  const [filterPatterns, setFilterPatterns] = useState<string[]>([]);
  const [description, setDescription] = useState<string>('');
  const [running, setRunning] = useState(false);

  const isValid = useMemo(() => {
    if (!description.trim()) return false;
    if ((filterMode === 'whitelist' || filterMode === 'blacklist') && filterPatterns.length === 0) {
      return false;
    }
    if (filterMode === 'all' && filterPatterns.length > 0) {
      return false;
    }
    return true;
  }, [description, filterMode, filterPatterns]);

  const doRun = async () => {
    // mirror backend rules
    if (!description.trim()) {
      message.warning('Please enter a short description for this run.');
      return;
    }
    if ((filterMode === 'whitelist' || filterMode === 'blacklist') && !filterPatterns.length) {
      message.warning('Please add at least one file pattern for whitelist/blacklist.');
      return;
    }
    if (filterMode === 'all' && filterPatterns.length) {
      message.warning('“All files” ignores patterns — clear the patterns or choose another mode.');
      return;
    }

    setRunning(true);
    try {
      const payload: {
        description: string;
        filter_mode: MossFilterMode;
        filter_patterns?: string[];
      } = {
        description: description.trim(),
        filter_mode: filterMode,
      };
      if (filterMode !== 'all') payload.filter_patterns = filterPatterns;

      const res = await runMossCheck(moduleId, assignmentId, payload);
      if (res.success) {
        message.success(res.message || 'Started MOSS job');
        onClose();
        onRan?.();
      } else {
        message.error(res.message || 'Failed to start MOSS job');
      }
    } catch {
      message.error('Failed to start MOSS job');
    } finally {
      setRunning(false);
    }
  };

  return (
    <Modal
      title="Run MOSS on Latest Submissions"
      open={open}
      onCancel={onClose}
      width={650}
      onOk={doRun}
      okText="Run MOSS"
      confirmLoading={running}
      okButtonProps={{ disabled: !isValid }}
      getContainer={false}
    >
      <Space direction="vertical" className="w-full">
        <Typography.Paragraph className="mb-1">
          <strong>MOSS checks for plagiarism between student submissions.</strong> It compares one
          submission per student based on the assignment’s <em>Grading Policy</em>.
        </Typography.Paragraph>

        <Alert
          type="warning"
          showIcon
          message="MOSS uses the language from Assignment Config"
          description={<span>Ensure the correct language is set before running.</span>}
        />

        <Alert
          type="info"
          showIcon
          message="Which submissions are compared?"
          description={
            <div>
              One submission per student based on <em>Grading Policy</em>:
              <ul className="list-disc ml-6 mt-2">
                <li>
                  <strong>Last</strong>: most recent <em>non-practice</em>, <em>non-ignored</em>.
                </li>
                <li>
                  <strong>Best</strong>: highest-scoring <em>non-practice</em>, <em>non-ignored</em>
                  .
                </li>
              </ul>
            </div>
          }
        />

        <Divider className="!my-2" />

        <div className="inline-flex items-center gap-1">
          <Typography.Text strong className="!mb-0">
            Report Description <span className="text-red-500">*</span>
          </Typography.Text>
          <Tooltip title="This note is saved with the report and shown in the list.">
            <InfoCircleOutlined className="text-gray-400 align-middle cursor-help" />
          </Tooltip>
        </div>
        <Input.TextArea
          value={description}
          onChange={(e) => setDescription(e.target.value)}
          placeholder="e.g. Week 3 — ignore skeleton/ and **/*.test.cpp"
          rows={2}
          maxLength={240}
          showCount
          status={!description.trim() ? 'error' : undefined}
        />

        <Divider className="!my-2" />

        <div className="inline-flex items-center gap-1">
          <Typography.Text strong className="!mb-0">
            File filters
          </Typography.Text>
          <Tooltip
            title={
              <div>
                Choose which files MOSS should consider:
                <ul className="list-disc ml-4 mt-1">
                  <li>
                    <b>All</b>: compare every file.
                  </li>
                  <li>
                    <b>Whitelist</b>: only files matching patterns.
                  </li>
                  <li>
                    <b>Blacklist</b>: exclude files matching patterns.
                  </li>
                </ul>
                Patterns use globbing, e.g. <code>**/*.cpp</code>, <code>src/**</code>,{' '}
                <code>main.java</code>.
              </div>
            }
          >
            <InfoCircleOutlined className="text-gray-400 align-middle cursor-help" />
          </Tooltip>
        </div>

        <Radio.Group
          value={filterMode}
          onChange={(e) => setFilterMode(e.target.value)}
          options={MOSS_FILTER_MODES.map((m) => ({
            label: m === 'all' ? 'All files' : m[0].toUpperCase() + m.slice(1),
            value: m,
          }))}
          optionType="button"
          buttonStyle="solid"
        />

        <div className="mt-2">
          <Typography.Text type="secondary">Patterns</Typography.Text>
          <Select
            mode="tags"
            value={filterPatterns}
            onChange={(vals) => setFilterPatterns(vals as string[])}
            placeholder={
              filterMode === 'all'
                ? 'No patterns needed for “All files”'
                : 'Add patterns, e.g. **/*.cpp, src/**, main.java'
            }
            disabled={filterMode === 'all'}
            className="w-full mt-1"
            tokenSeparators={[',', ' ']}
          />
          {filterMode !== 'all' && (
            <Typography.Text type="secondary" className="text-xs">
              Whitelist: only these files are compared. Blacklist: these files are excluded.
            </Typography.Text>
          )}
        </div>

        {/* Removed the “latest report generated / archived” footer lines */}
      </Space>
    </Modal>
  );
};

export default MossRunModal;
