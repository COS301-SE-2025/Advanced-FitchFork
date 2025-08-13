import React, { useState } from 'react';
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

  const auth = useAuth();
  const module = useModule();
  const isStudent = auth.isStudent(module.id);

  const handleViewDiff = (taskName: string, taskNumber: number) => {
    const expected = memoOutput.find((m) => m.task_number === taskNumber)?.raw ?? '';
    const actual = submisisonOutput.find((s) => s.task_number === taskNumber)?.raw ?? '';

    setCurrentTask({ name: taskName, expected, actual });
    setVisible(true);
  };

  const collapseItems: CollapseProps['items'] = tasks.map((task) => {
    const { task_number, name, score, subsections } = task;

    const children = (
      <ul className="space-y-2 pl-6">
        {subsections.map((sub, idx) => (
          <li
            key={idx}
            className="flex items-center text-sm text-neutral-700 dark:text-neutral-300"
          >
            <div className="flex-1 flex items-center gap-1">
              <Tag color={getScoreTagColor(sub.earned, sub.total)}>
                {sub.earned} / {sub.total}
              </Tag>
              <span>{sub.label}</span>
            </div>
          </li>
        ))}
      </ul>
    );

    const label = (
      <div className="flex items-center gap-2">
        <Tag color={getScoreTagColor(score.earned, score.total)}>
          {score.earned} / {score.total}
        </Tag>
        <Text className="font-medium">{name}</Text>
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

    return {
      key: task_number,
      label,
      children,
      extra,
    };
  });

  return (
    <>
      <Collapse
        bordered
        items={collapseItems}
        defaultActiveKey={[String(tasks[0]?.task_number)]}
        className="rounded-md"
      />

      <Modal
        open={visible}
        onCancel={() => setVisible(false)}
        footer={null}
        width={1400}
        title={`Output Difference for ${currentTask?.name}`}
        className="!p-0"
      >
        <Typography.Paragraph type="secondary" className="mb-4 text-sm">
          <strong>Note:</strong> The submission output is shown on the <strong>left</strong>, and
          the memo (expected) output is on the <strong>right</strong>.
        </Typography.Paragraph>

        <CodeDiffEditor
          title={<Text strong>Student vs Memo</Text>}
          original={currentTask?.actual ?? ''}
          modified={currentTask?.expected ?? ''}
          showLangBadge={false}
          language="plaintext"
          height={600}
          minimal={false}
          leftTitle={<Text strong>Student Output</Text>}
          rightTitle={<Text strong>Memo Output</Text>}
        />
      </Modal>
    </>
  );
};

export default SubmissionTasks;
