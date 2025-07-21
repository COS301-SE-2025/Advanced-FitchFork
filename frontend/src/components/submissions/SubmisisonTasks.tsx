import React, { useState } from 'react';
import { Collapse, Tag, Typography, Modal, Button } from 'antd';
import type { CollapseProps } from 'antd';
import type { TaskBreakdown } from '@/types/modules/assignments/submissions';
import CodeDiffEditor from '@/components/CodeDiffEditor';

const { Text } = Typography;

type Props = {
  tasks: TaskBreakdown[];
};

const getScoreTagColor = (earned: number, total: number): string => {
  if (total === 0) return 'default';
  const percent = (earned / total) * 100;
  if (percent >= 85) return 'green';
  if (percent >= 50) return 'orange';
  return 'red';
};

const SubmissionTasks: React.FC<Props> = ({ tasks }) => {
  const [visible, setVisible] = useState(false);
  const [currentLabel, setCurrentLabel] = useState<string>('');

  const handleViewDiff = (label: string) => {
    setCurrentLabel(label);
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

            {sub.earned !== sub.total && (
              <Button type="link" size="small" onClick={() => handleViewDiff(sub.label)}>
                View Diff
              </Button>
            )}
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
      score.earned !== score.total ? (
        <Button
          type="link"
          size="small"
          onClick={(e) => {
            e.stopPropagation();
            handleViewDiff(name);
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
        width={1000}
        title={`Diff for: ${currentLabel}`}
      >
        <CodeDiffEditor
          title={currentLabel}
          original={`Expected output of ${currentLabel}\nLine 2\nLine 3`}
          modified={`Actual output of ${currentLabel}\nLine 2 wrong\nLine 3`}
          language="plaintext"
          height={400}
        />
      </Modal>
    </>
  );
};

export default SubmissionTasks;
