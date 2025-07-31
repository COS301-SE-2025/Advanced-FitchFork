import { useEffect, useState } from 'react';
import { Typography, Button, Input, Spin, Empty, message } from 'antd';
import { PlusOutlined, SaveOutlined, EditOutlined, DeleteOutlined } from '@ant-design/icons';

import { useModule } from '@/context/ModuleContext';
import { useAssignmentSetup } from '@/context/AssignmentSetupContext';
import { listTasks, createTask, editTask, deleteTask } from '@/services/modules/assignments/tasks';

const { Title, Paragraph, Text } = Typography;

const StepTasks = () => {
  const module = useModule();
  const { assignmentId, refreshAssignment, onStepComplete } = useAssignmentSetup();

  const [tasks, setTasks] = useState<
    { id: number; task_number: number; name: string; command: string }[]
  >([]);
  const [loading, setLoading] = useState(false);

  const [editingTaskId, setEditingTaskId] = useState<number | null>(null);
  const [editedName, setEditedName] = useState('');
  const [editedCommand, setEditedCommand] = useState('');

  const fetchTasks = async () => {
    if (!assignmentId) return;
    setLoading(true);
    const res = await listTasks(module.id, assignmentId);
    if (res.success) {
      setTasks(res.data.sort((a, b) => a.task_number - b.task_number));
    } else {
      message.error(res.message);
    }
    setLoading(false);
  };

  useEffect(() => {
    fetchTasks();
  }, [assignmentId]);

  const handleCreateTask = async () => {
    if (!assignmentId) return;

    const nextTaskNumber = (tasks[tasks.length - 1]?.task_number ?? 0) + 1;

    const res = await createTask(module.id, assignmentId, {
      task_number: nextTaskNumber,
      name: `Task ${nextTaskNumber}`,
      command: 'echo Hello World',
    });

    if (res.success) {
      message.success('Task created');
      await fetchTasks();
      await refreshAssignment?.();
      onStepComplete?.();
    } else {
      message.error(res.message);
    }
  };

  const handleSaveTask = async (taskId: number) => {
    if (!assignmentId) return;

    const res = await editTask(module.id, assignmentId, taskId, {
      name: editedName,
      command: editedCommand,
    });

    if (res.success) {
      message.success('Task updated');
      await fetchTasks();
      await refreshAssignment?.();
      setEditingTaskId(null);
    } else {
      message.error(res.message);
    }
  };

  const handleDeleteTask = async (taskId: number) => {
    if (!assignmentId) return;

    try {
      const res = await deleteTask(module.id, assignmentId, taskId);

      if (res.success) {
        message.success(res.message || 'Task deleted');
        await fetchTasks();
        await refreshAssignment?.();
        onStepComplete?.();
      } else {
        message.error(res.message || 'Failed to delete task');
      }
    } catch (err) {
      message.error('Failed to delete task');
      console.error(err);
    }
  };

  return (
    <div className="space-y-6">
      <Title level={3}>Define Tasks</Title>
      <Paragraph type="secondary">
        Below are the tasks that make up this assignment. You can edit a task inline or add new
        ones.
      </Paragraph>

      <div className="flex justify-end mb-4">
        <Button icon={<PlusOutlined />} type="primary" onClick={handleCreateTask}>
          Add Task
        </Button>
      </div>

      {loading ? (
        <Spin />
      ) : tasks.length === 0 ? (
        <Empty description="No tasks yet. Add one to get started." />
      ) : (
        <div className="space-y-2">
          {tasks.map((task) => (
            <div
              key={task.id}
              data-cy="task-card"
              className="p-3 bg-white dark:bg-gray-900 border border-gray-200 dark:border-gray-700 rounded flex flex-col md:flex-row md:items-center md:justify-between gap-2"
            >
              <div className="flex flex-col md:flex-row md:gap-4">
                {editingTaskId === task.id ? (
                  <>
                    <Input
                      value={editedName}
                      onChange={(e) => setEditedName(e.target.value)}
                      placeholder="Task Name"
                      className="w-48"
                    />
                    <Input
                      value={editedCommand}
                      onChange={(e) => setEditedCommand(e.target.value)}
                      placeholder="Command"
                      className="w-72"
                    />
                  </>
                ) : (
                  <div className="flex flex-col md:flex-row md:gap-4">
                    <Text className="text-base font-medium">{task.name}</Text>
                    <span className="text-xs font-mono bg-gray-100 dark:bg-gray-800 text-gray-700 dark:text-gray-200 px-2 py-1 rounded">
                      {task.command}
                    </span>
                  </div>
                )}
              </div>

              <div className="flex gap-2">
                {editingTaskId === task.id ? (
                  <Button
                    icon={<SaveOutlined />}
                    type="primary"
                    size="small"
                    onClick={() => handleSaveTask(task.id)}
                    style={{ height: 32 }}
                  >
                    Save
                  </Button>
                ) : (
                  <Button
                    icon={<EditOutlined />}
                    size="small"
                    onClick={() => {
                      setEditingTaskId(task.id);
                      setEditedName(task.name);
                      setEditedCommand(task.command);
                    }}
                    style={{ height: 32 }}
                  >
                    Edit
                  </Button>
                )}

                <Button
                  icon={<DeleteOutlined />}
                  size="small"
                  danger
                  onClick={() => handleDeleteTask(task.id)}
                  style={{ height: 32 }}
                >
                  Delete
                </Button>
              </div>
            </div>
          ))}
        </div>
      )}
    </div>
  );
};

export default StepTasks;
