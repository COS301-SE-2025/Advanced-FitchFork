import { Typography, Collapse } from 'antd';
import { useAssignment } from '@/context/AssignmentContext';
import CodeEditor from '@/components/CodeEditor';
const { Text } = Typography;
const { Panel } = Collapse;

const MemoOutput = () => {
  const { memoOutput } = useAssignment();

  return (
    <div className="space-y-6">
      {memoOutput.length === 0 ? (
        <Text type="secondary">No memo output available for this assignment.</Text>
      ) : (
        <Collapse accordion className="!bg-white dark:!bg-gray-950 !rounded-lg">
          {memoOutput.map((task) => {
            const combinedOutput = task.subsections
              .map((sub) => `# ${sub.label}\n${sub.output}`)
              .join('\n\n');

            return (
              <Panel
                key={task.task_number}
                className="dark:!bg-gray-900"
                header={
                  <span className="font-medium text-gray-900 dark:text-gray-100">{task.name}</span>
                }
              >
                <CodeEditor
                  title="Output"
                  value={combinedOutput}
                  language="text"
                  readOnly
                  height={400}
                  className="rounded-md"
                />
              </Panel>
            );
          })}
        </Collapse>
      )}
    </div>
  );
};

export default MemoOutput;
