import React, { useMemo, useState } from 'react';
import { Collapse, Tag, Typography, Modal, Button } from 'antd';
import type { CollapseProps } from 'antd';
import type { SubmissionTaskOutput, TaskBreakdown } from '@/types/modules/assignments/submissions';
import CodeDiffEditor from '@/components/common/CodeDiffEditor';
import { useAuth } from '@/context/AuthContext';
import { useModule } from '@/context/ModuleContext';
import type { MemoTaskOutput } from '@/types/modules/assignments/memo-output';

const { Text } = Typography;

type Props = {
  tasks: TaskBreakdown[];
  memoOutput: MemoTaskOutput[];
  submisisonOutput: SubmissionTaskOutput[];
};

const getScoreTagColor = (earned: number, total: number): string => {
  if (total === 0) return 'default';
  const percent = (earned / total) * 100;
  if (percent >= 85) return 'green';
  if (percent >= 50) return 'orange';
  return 'red';
};

const SubmissionTasks: React.FC<Props> = ({ tasks, memoOutput, submisisonOutput }) => {
  const [visible, setVisible] = useState(false);
  const [currentTask, setCurrentTask] = useState<{
    name: string;
    expected: string;
    actual: string;
  } | null>(null);

  // simple: which subsection feedbacks are expanded (for full marks & 0/0)
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

  const collapseItems: CollapseProps['items'] = useMemo(
    () =>
      tasks.map((task, task_idx) => {
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

        const extra =
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

        return { key: task_number, label, children, extra };
      }),
    [tasks, isStudent, expandedFeedback],
  );

  return (
    <>
      <Collapse bordered items={collapseItems} className="rounded-md" defaultActiveKey={[]} />

      <Modal
        open={visible}
        onCancel={() => setVisible(false)}
        footer={null}
        centered={false}
        width="calc(100vw - 48px)" // ← leaves 24px gap left/right
        style={{ top: 24 }} // ← leaves 24px gap top
        title={`Output Difference for ${currentTask?.name ?? 'Task'}`}
        className="!p-0"
        styles={{
          content: {
            height: 'calc(100vh - 48px)', // ← leaves 24px gap bottom
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
