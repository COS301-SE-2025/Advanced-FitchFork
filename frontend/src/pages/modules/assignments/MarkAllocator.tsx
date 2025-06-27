import { useEffect, useState } from 'react';
import { Typography, Spin, InputNumber, Collapse, Alert, Button } from 'antd';
import { useModule } from '@/context/ModuleContext';
import { useAssignment } from '@/context/AssignmentContext';

import {
  getMarkAllocator,
  updateMarkAllocator,
} from '@/services/modules/assignments/mark-allocator';
import type {
  MarkAllocatorItem,
  MarkAllocatorTask,
} from '@/types/modules/assignments/mark-allocator';
import { useNotifier } from '@/components/Notifier';

const { Text } = Typography;
const { Panel } = Collapse;

const MarkAllocator = () => {
  const module = useModule();
  const { assignment } = useAssignment();
  const { notifySuccess, notifyError } = useNotifier();

  const [allocator, setAllocator] = useState<MarkAllocatorItem[]>([]);
  const [loading, setLoading] = useState(true);
  const [saving, setSaving] = useState(false);

  useEffect(() => {
    if (!module.id || !assignment.id) return;

    getMarkAllocator(module.id, assignment.id)
      .then((res) => {
        if (res.success && res.data) {
          console.log(res.data.tasks);
          setAllocator(res.data.tasks);
        } else {
          setAllocator([]); // fallback to empty array to prevent crash
          notifyError('Failed to load mark allocator', res.message);
        }
      })
      .catch((err) => {
        console.error(err);
        setAllocator([]); // fallback
        notifyError('Error', 'An unexpected error occurred');
      })
      .finally(() => setLoading(false));
  }, [module.id, assignment.id]);

  const handleSave = async () => {
    setSaving(true);
    try {
      const res = await updateMarkAllocator(module.id, assignment.id, allocator);
      if (res.success) {
        notifySuccess('Changes saved', 'Mark allocator updated successfully.');
      } else {
        notifyError('Failed to save changes', res.message);
      }
    } catch (err) {
      console.error(err);
      notifyError('Error', 'Unexpected error while saving.');
    } finally {
      setSaving(false);
    }
  };

  if (loading) {
    return <Spin tip="Loading mark allocator..." />;
  }

  if (allocator.length === 0) {
    return <Alert type="info" message="No mark allocator found for this assignment." />;
  }

  return (
    <div className="max-w-4xl space-y-6">
      <div className="pb-1">
        <Text type="secondary">
          This page displays how marks are allocated across tasks and their subsections. You can
          edit the values and save changes.
        </Text>
      </div>

      <Collapse accordion className="!bg-white dark:!bg-gray-950 !rounded-lg">
        {allocator.map((item, index) => {
          const taskKey = Object.keys(item)[0];
          const task: MarkAllocatorTask = item[taskKey];

          return (
            <Panel
              key={index}
              header={
                <span className="font-medium text-gray-900 dark:text-gray-100">{task.name}</span>
              }
              className="!bg-gray-50 dark:!bg-gray-900"
            >
              <div>
                <div className="flex justify-between items-center mb-4">
                  <Text strong>Total Task Marks</Text>
                  <InputNumber
                    min={0}
                    value={task.value}
                    onChange={(val) => {
                      const updated = [...allocator];
                      updated[index][taskKey].value = val || 0;
                      setAllocator(updated);
                    }}
                  />
                </div>

                {task.subsections && task.subsections.length > 0 && (
                  <div className="space-y-3">
                    {task.subsections.map((sub, subIndex) => (
                      <div key={subIndex} className="flex justify-between items-center">
                        <Text>{sub.name}</Text>
                        <InputNumber
                          min={0}
                          value={sub.value}
                          onChange={(val) => {
                            const updated = [...allocator];
                            updated[index][taskKey].subsections![subIndex].value = val || 0;
                            setAllocator(updated);
                          }}
                        />
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
        <Button type="primary" loading={saving} onClick={handleSave}>
          Save Changes
        </Button>
      </div>
    </div>
  );
};

export default MarkAllocator;
