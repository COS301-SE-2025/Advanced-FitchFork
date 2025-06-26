import { useEffect, useState } from 'react';
import { Typography, Spin, Collapse, Button } from 'antd';
import { CopyOutlined } from '@ant-design/icons';
import { useModule } from '@/context/ModuleContext';
import { useAssignment } from '@/context/AssignmentContext';
import { useNotifier } from '@/components/Notifier';
import { getMemoOutput } from '@/services/modules/assignments/memo-output';
import type { MemoTaskOutput } from '@/types/modules/assignments/memo-output/shared';

const { Text, Paragraph } = Typography;
const { Panel } = Collapse;

const MemoOutput = () => {
  const module = useModule();
  const { assignment } = useAssignment();
  const { notifyError } = useNotifier();

  const [loading, setLoading] = useState(true);
  const [outputs, setOutputs] = useState<MemoTaskOutput[]>([]);

  useEffect(() => {
    if (!module.id || !assignment.id) return;

    getMemoOutput(module.id, assignment.id)
      .then((res) => {
        if (res.success && res.data) {
          setOutputs(res.data);
        } else {
          notifyError('Failed to load memo output', res.message);
        }
      })
      .catch((err) => {
        console.error(err);
        notifyError('Error', 'An unexpected error occurred.');
      })
      .finally(() => setLoading(false));
  }, [module.id, assignment.id]);

  if (loading) {
    return <Spin tip="Loading memo output..." />;
  }

  return (
    <div className="max-w-6xl space-y-4">
      <div className="mb-4">
        <Text type="secondary">
          The following is the output generated for this assignment's memo sections. Each block
          corresponds to a task. You can copy individual sections or the full task output using the
          buttons.
        </Text>
      </div>

      {outputs.length === 0 ? (
        <Text type="secondary">No memo output available for this assignment.</Text>
      ) : (
        <Collapse accordion className="!bg-white dark:!bg-gray-950 !rounded-lg">
          {outputs.map((task) => (
            <Panel
              key={task.task_number}
              className="!bg-gray-50 dark:!bg-gray-900"
              header={
                <div className="flex justify-between items-center">
                  <span className="font-medium text-gray-900 dark:text-gray-100">{task.name}</span>
                  <Button
                    icon={<CopyOutlined />}
                    size="small"
                    onClick={(e) => {
                      e.stopPropagation(); // prevent collapsing
                      navigator.clipboard.writeText(task.raw);
                    }}
                  >
                    Copy All
                  </Button>
                </div>
              }
            >
              <div className="space-y-6">
                {task.subsections.map((sub, idx) => (
                  <div
                    key={idx}
                    className="!bg-gray-100 dark:!bg-gray-800 !border !border-gray-300 dark:!border-gray-700 !rounded-md !p-4"
                  >
                    <div className="flex justify-between items-center mb-2">
                      <div className="font-medium text-gray-900 dark:text-gray-100">
                        {sub.label}
                      </div>
                      <Button
                        icon={<CopyOutlined />}
                        size="small"
                        onClick={() => navigator.clipboard.writeText(sub.output)}
                      >
                        Copy
                      </Button>
                    </div>
                    <Paragraph className="text-sm whitespace-pre-wrap text-gray-800 dark:text-gray-200">
                      {sub.output}
                    </Paragraph>
                  </div>
                ))}
              </div>
            </Panel>
          ))}
        </Collapse>
      )}
    </div>
  );
};

export default MemoOutput;
