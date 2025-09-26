import React, { useMemo, useState } from 'react';
import { Collapse, Tag, Typography, Modal, Button, Tooltip } from 'antd';
import type { CollapseProps } from 'antd';
import type {
  CodeCoverage,
  SubmissionTaskOutput,
  TaskBreakdown,
} from '@/types/modules/assignments/submissions';
import CodeDiffEditor from '@/components/common/CodeDiffEditor';
import { useAuth } from '@/context/AuthContext';
import { useModule } from '@/context/ModuleContext';
import type { MemoTaskOutput } from '@/types/modules/assignments/memo-output';

const { Text } = Typography;

type ItemsType = NonNullable<CollapseProps['items']>;
type ItemType = ItemsType[number];

type Props = {
  tasks: TaskBreakdown[];
  memoOutput: MemoTaskOutput[];
  submisisonOutput: SubmissionTaskOutput[];
  codeCoverage?: CodeCoverage;
};

const getScoreTagColor = (earned: number, total: number): string => {
  if (total === 0) return 'default';
  const percent = (earned / total) * 100;
  if (percent >= 85) return 'green';
  if (percent >= 50) return 'orange';
  return 'red';
};

const SubmissionTasks: React.FC<Props> = ({
  tasks,
  memoOutput,
  submisisonOutput,
  codeCoverage,
}) => {
  const [visible, setVisible] = useState(false);
  const [currentTask, setCurrentTask] = useState<{
    name: string;
    expected: string;
    actual: string;
  } | null>(null);

  // which subsection feedbacks are expanded (for full marks & 0/0)
  const [expandedFeedback, setExpandedFeedback] = useState<Set<string>>(new Set());
  const toggleFeedback = (taskNumber: number, subIdx: number) => {
    const key = `${taskNumber}:${subIdx}`;
    setExpandedFeedback((prev) => {
      const next = new Set(prev);
      next.has(key) ? next.delete(key) : next.add(key);
      return next;
    });
  };

  const auth = useAuth();
  const module = useModule();
  const isStudent = auth.isStudent(module.id);

  const handleViewDiff = (taskName: string, taskNumber: number) => {
    const expected = memoOutput.find((m) => m.task_number === taskNumber)?.raw ?? '';
    const actual = submisisonOutput.find((s) => s.task_number === taskNumber)?.raw ?? '';
    setCurrentTask({ name: taskName, expected, actual });
    setVisible(true);
  };

  // Coverage panel: now uses summary.coverage_percent, total_lines, covered_lines
  const coverageItem: ItemType | null = useMemo(() => {
    if (!codeCoverage) return null;

    const summary = codeCoverage.summary;
    const files = codeCoverage.files ?? [];

    const pctText =
      summary && Number.isFinite(summary.coverage_percent)
        ? `${Math.round(summary.coverage_percent)}%`
        : '—';

    const linesText =
      summary && Number.isFinite(summary.covered_lines) && Number.isFinite(summary.total_lines)
        ? `${summary.covered_lines}/${summary.total_lines} lines`
        : '—';

    const fileList = files.length ? (
      <ul className="space-y-1 pl-0">
        {files.map((f, i) => {
          const filePct = f.total > 0 ? Math.round((f.earned / f.total) * 100) : null;
          return (
            <li key={`${f.path}-${i}`} className="flex items-center gap-2 text-sm">
              <Tag color={getScoreTagColor(f.earned, f.total)}>
                {f.earned}/{f.total}
              </Tag>
              <span className="truncate">{f.path}</span>
              <span className="ml-auto text-xs text-gray-500">
                {filePct !== null ? `${filePct}%` : '—'}
              </span>
            </li>
          );
        })}
      </ul>
    ) : (
      <Text type="secondary">No per-file details.</Text>
    );

    const labelLeft = (
      <Tag color={summary ? getScoreTagColor(summary.earned, summary.total) : 'default'}>
        {summary ? `${summary.earned} / ${summary.total}` : '—'}
      </Tag>
    );

    const labelMiddle = (
      <div className="flex items-center gap-2 min-w-0">
        <Text className="font-medium truncate">Code Coverage</Text>
        {summary ? (
          <Tooltip title={`${linesText} covered`}>
            <Text type="secondary" className="shrink-0">
              ({linesText})
            </Text>
          </Tooltip>
        ) : null}
      </div>
    );

    const labelRight = (
      <Text type="secondary" className="ml-auto shrink-0">
        {pctText}
      </Text>
    );

    const item: ItemType = {
      key: -1, // numeric & non-colliding (tasks are positive)
      label: (
        <div className="flex items-center gap-2 min-w-0 w-full">
          {labelLeft}
          {labelMiddle}
          {labelRight}
        </div>
      ),
      children: <div className="pl-2">{fileList}</div>,
      extra: null,
    };

    return item;
  }, [codeCoverage]);

  const collapseItems: ItemsType = useMemo(() => {
    const items: ItemsType = tasks.map((task, task_idx) => {
      const { task_number, name, score, subsections } = task;

      const children = (
        <ul className="space-y-2 pl-6">
          {subsections.map((sub, idx) => {
            const isFullMarks = (sub.total > 0 && sub.earned === sub.total) || sub.total === 0;
            const hasFeedback = !!sub.feedback?.trim();
            const key = `${task_number}:${idx}`;
            const showFeedback = hasFeedback && (!isFullMarks || expandedFeedback.has(key));

            return (
              <li key={idx} className="text-sm">
                <div className="flex items-center text-neutral-800 dark:text-neutral-200">
                  <Tag color={getScoreTagColor(sub.earned, sub.total)} className="mr-2">
                    {sub.earned} / {sub.total}
                  </Tag>
                  <span className="truncate">{sub.label}</span>

                  {hasFeedback && isFullMarks && (
                    <Button
                      type="link"
                      size="small"
                      className="!ml-auto !px-0"
                      onClick={() => toggleFeedback(task_number, idx)}
                    >
                      {expandedFeedback.has(key) ? 'Hide Feedback' : 'Show Feedback'}
                    </Button>
                  )}
                </div>

                {showFeedback && (
                  <pre
                    className="!mt-1 whitespace-pre-wrap text-xs font-mono
                               bg-neutral-100 dark:bg-neutral-800
                               text-neutral-700 dark:text-neutral-300
                               rounded p-2 leading-snug"
                  >
                    {sub.feedback}
                  </pre>
                )}
              </li>
            );
          })}
        </ul>
      );

      const label = (
        <div className="flex items-center gap-2 min-w-0">
          <Tag color={getScoreTagColor(score.earned, score.total)}>
            {score.earned} / {score.total}
          </Tag>
          <Text className="font-medium truncate">{name?.trim() || `Task ${task_idx + 1}`}</Text>
        </div>
      );

      const extra: React.ReactNode =
        !isStudent && score.earned !== score.total ? (
          <Button
            type="link"
            size="small"
            onClick={(e) => {
              e.stopPropagation();
              handleViewDiff(name, task_number);
            }}
          >
            View Diff
          </Button>
        ) : null;

      const item: ItemType = { key: task_number, label, children, extra };
      return item;
    });

    if (coverageItem) items.push(coverageItem); // put coverage LAST
    return items;
  }, [tasks, isStudent, expandedFeedback, coverageItem]);

  return (
    <>
      <Collapse bordered items={collapseItems} className="rounded-md" defaultActiveKey={[]} />

      <Modal
        open={visible}
        onCancel={() => setVisible(false)}
        footer={null}
        centered={false}
        width="calc(100vw - 48px)"
        style={{ top: 24 }}
        title={`Output Difference for ${currentTask?.name ?? 'Task'}`}
        className="!p-0"
        styles={{
          content: {
            height: 'calc(100vh - 48px)',
            maxWidth: '100%',
            padding: 0,
            display: 'flex',
            flexDirection: 'column',
          },
          header: { padding: '12px 16px' },
          body: {
            padding: '8px 16px 16px',
            display: 'flex',
            flexDirection: 'column',
            flex: 1,
            minHeight: 0,
          },
        }}
      >
        <div className="flex-1 min-h-0">
          <CodeDiffEditor
            original={currentTask?.actual ?? ''}
            modified={currentTask?.expected ?? ''}
            language="plaintext"
            className="h-full"
            leftTitle={<Text strong>Student Output</Text>}
            rightTitle={<Text strong>Memo Output</Text>}
          />
        </div>
      </Modal>
    </>
  );
};

export default SubmissionTasks;
