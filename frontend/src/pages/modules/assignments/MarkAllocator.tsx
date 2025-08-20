import { useEffect } from 'react';
import { Typography, InputNumber, Collapse, Alert, Button } from 'antd';

import { useAssignment } from '@/context/AssignmentContext';
import { useBreadcrumbContext } from '@/context/BreadcrumbContext';

const { Text } = Typography;
const { Panel } = Collapse;

const MarkAllocator = () => {
  const { markAllocator } = useAssignment();
  const { customLabels, setBreadcrumbLabel } = useBreadcrumbContext();

  useEffect(() => {
    const pathKey = location.pathname.replace(/^\//, '');
    if (customLabels[pathKey] !== 'Mark Allocator') {
      setBreadcrumbLabel(pathKey, 'Mark Allocator');
    }
  }, [location.pathname, customLabels, setBreadcrumbLabel]);

  if (!markAllocator) {
    return <Alert type="info" message="No mark allocator found for this assignment." />;
  }

  return (
    <div>
      <Collapse accordion className="!bg-white dark:!bg-gray-950 !rounded-none !border-x-0">
        {markAllocator.tasks.map((taskWrapper, index) => {
          const taskKey = Object.keys(taskWrapper)[0];
          const task = taskWrapper[taskKey];

          return (
            <Panel
              key={index}
              header={
                <span className="font-medium text-gray-900 dark:text-gray-100">{task.name}</span>
              }
              className="dark:!bg-gray-900"
            >
              <div>
                <div className="flex justify-between items-center mb-4">
                  <Text strong>Total Task Marks</Text>
                  <InputNumber min={0} value={task.value} />
                </div>

                {task.subsections && task.subsections.length > 0 && (
                  <div className="space-y-3">
                    {task.subsections.map((sub, subIndex) => (
                      <div key={subIndex} className="flex justify-between items-center">
                        <Text>{sub.name}</Text>
                        <InputNumber min={0} value={sub.value} />
                      </div>
                    ))}
                  </div>
                )}
              </div>
            </Panel>
          );
        })}
      </Collapse>

      <div className="pt-4">
        <Button type="primary">Save Changes</Button>
      </div>
    </div>
  );
};

export default MarkAllocator;
