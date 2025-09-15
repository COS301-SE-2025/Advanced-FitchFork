import { useEffect, useState } from 'react';
import {
  Typography,
  Button,
  Input,
  Spin,
  Empty,
  message,
  Upload,
  Switch,
  Tooltip,
  Space,
} from 'antd';
import {
  PlusOutlined,
  SaveOutlined,
  EditOutlined,
  DeleteOutlined,
  UploadOutlined,
} from '@ant-design/icons';
import Tip from '@/components/common/Tip';

import { useModule } from '@/context/ModuleContext';
import { useAssignmentSetup } from '@/context/AssignmentSetupContext';
import { listTasks, createTask, editTask, deleteTask } from '@/services/modules/assignments/tasks';
import { uploadOverwriteFiles } from '@/services/modules/assignments/overwrite_files/post';

const { Title, Paragraph, Text } = Typography;

type TaskRow = {
  id: number;
  task_number: number;
  name: string;
  command: string;
  code_coverage: boolean;
};

const StepTasks = () => {
  const module = useModule();
  const { assignmentId, refreshAssignment, setStepSaveHandler } = useAssignmentSetup();

  const [tasks, setTasks] = useState<TaskRow[]>([]);
  const [loading, setLoading] = useState(false);

  const [editingTaskId, setEditingTaskId] = useState<number | null>(null);
  const [editedName, setEditedName] = useState('');
  const [editedCommand, setEditedCommand] = useState('');
  const [savingId, setSavingId] = useState<number | null>(null);
  const [uploadingFor, setUploadingFor] = useState<number | null>(null);

  const fetchTasks = async () => {
    if (!assignmentId) return;
    setLoading(true);
    try {
      const res = await listTasks(module.id, assignmentId);
      if (res.success) {
        setTasks(res.data.sort((a: TaskRow, b: TaskRow) => a.task_number - b.task_number));
      } else {
        message.error(res.message || 'Failed to load tasks');
      }
    } finally {
      setLoading(false);
    }
  };

  useEffect(() => {
    fetchTasks();
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [assignmentId]);

  // Register a no-op save handler for step 3 (Tasks)
  useEffect(() => {
    setStepSaveHandler?.(3, async () => true);
  }, [setStepSaveHandler]);

  const handleCreateTask = async () => {
    if (!assignmentId) return;

    const nextTaskNumber = (tasks[tasks.length - 1]?.task_number ?? 0) + 1;

    const res = await createTask(module.id, assignmentId, {
      task_number: nextTaskNumber,
      name: `Task ${nextTaskNumber}`,
      command: 'echo Hello World',
      code_coverage: false,
    });

    if (res.success) {
      await fetchTasks();
      await refreshAssignment?.();
    } else {
      message.error(res.message || 'Failed to create task');
    }
  };

  const handleSaveTask = async (taskId: number) => {
    if (!assignmentId) return;
    if (savingId === taskId) return; // prevent double-save

    const original = tasks.find((t) => t.id === taskId);
    if (!original) return;

    // no-op: nothing changed
    if (editedName.trim() === original.name && editedCommand.trim() === original.command) {
      setEditingTaskId(null);
      return;
    }

    try {
      setSavingId(taskId);
      const res = await editTask(module.id, assignmentId, taskId, {
        name: editedName.trim(),
        command: editedCommand.trim(),
        code_coverage: original?.code_coverage ?? false,
      });

      if (res.success) {
        await fetchTasks();
        await refreshAssignment?.();
        setEditingTaskId(null);
      } else {
        message.error(res.message || 'Failed to update task');
      }
    } finally {
      setSavingId(null);
    }
  };

  const handleDeleteTask = async (taskId: number) => {
    if (!assignmentId) return;

    try {
      const res = await deleteTask(module.id, assignmentId, taskId);

      if (res.success) {
        await fetchTasks();
        await refreshAssignment?.();
      } else {
        message.error(res.message || 'Failed to delete task');
      }
    } catch (err) {
      message.error('Failed to delete task');
      // eslint-disable-next-line no-console
      console.error(err);
    }
  };

  const handleToggleCoverage = async (task: TaskRow, value: boolean) => {
    if (!assignmentId) return;
    if (savingId === task.id) return;
    try {
      setSavingId(task.id);
      const res = await editTask(module.id, assignmentId, task.id, {
        name: task.name,
        command: task.command,
        code_coverage: value,
      });
      if (res.success) {
        setTasks((prev) =>
          prev.map((t) => (t.id === task.id ? { ...t, code_coverage: value } : t)),
        );
      } else {
        message.error(res.message || 'Failed to update coverage');
      }
    } finally {
      setSavingId(null);
    }
  };

  const beforeUploadOverwrite = (taskId: number) => async (file: File) => {
    if (!assignmentId) return false;
    try {
      setUploadingFor(taskId);
      const res = await uploadOverwriteFiles(module.id, assignmentId, taskId, [file]);
      if (res.success) {
        message.success('Overwrite files uploaded');
      } else {
        message.error(res.message || 'Upload failed');
      }
    } catch (e) {
      console.error(e);
      message.error('Upload failed');
    } finally {
      setUploadingFor(null);
    }
    return false; // prevent auto upload list
  };

  const beginEdit = (task: TaskRow) => {
    setEditingTaskId(task.id);
    setEditedName(task.name);
    setEditedCommand(task.command);
  };

  const cancelEdit = () => {
    setEditingTaskId(null);
    setEditedName('');
    setEditedCommand('');
  };

  return (
    <div className="space-y-6">
      <div className="flex items-center gap-2">
        <Title level={3} className="!mb-0">
          Define Tasks
        </Title>
        <Tip iconOnly newTab to="/help/assignments/tasks" text="Tasks help" />
      </div>
      <Paragraph type="secondary">
        Below are the tasks that make up this assignment. You can edit a task inline or add new
        ones.
      </Paragraph>

      {loading ? (
        <Spin />
      ) : tasks.length === 0 ? (
        <Empty description="No tasks yet. Add one to get started." />
      ) : (
        <div className="space-y-2">
          {tasks.map((task) => {
            const isEditing = editingTaskId === task.id;
            const isSaving = savingId === task.id;

            return (
              <div
                key={task.id}
                data-cy="task-card"
                className="p-3 bg-white dark:bg-gray-900 border border-gray-200 dark:border-gray-700 rounded flex flex-col md:flex-row md:items-center md:justify-between gap-2"
              >
                <div className="flex flex-col md:flex-row md:gap-4">
                  {isEditing ? (
                    <>
                      <Input
                        value={editedName}
                        onChange={(e) => setEditedName(e.target.value)}
                        placeholder="Task Name"
                        className="w-48"
                        disabled={isSaving}
                        onPressEnter={(e) => {
                          e.preventDefault();
                          e.stopPropagation();
                          void handleSaveTask(task.id);
                        }}
                        onKeyDown={(e) => {
                          if (e.key === 'Escape') cancelEdit();
                        }}
                      />
                      <Input
                        value={editedCommand}
                        onChange={(e) => setEditedCommand(e.target.value)}
                        placeholder="Command"
                        className="w-72"
                        disabled={isSaving}
                        onPressEnter={(e) => {
                          e.preventDefault();
                          e.stopPropagation();
                          void handleSaveTask(task.id);
                        }}
                        onKeyDown={(e) => {
                          if (e.key === 'Escape') cancelEdit();
                        }}
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

                <div className="flex items-center gap-3">
                  <div className="flex items-center gap-1 text-xs text-gray-600 dark:text-gray-300">
                    <Tooltip title="Mark this task as coverage-only (excluded from pass/fail)">
                      <span>Coverage</span>
                    </Tooltip>
                    <Switch
                      size="small"
                      checked={!!task.code_coverage}
                      loading={savingId === task.id}
                      onChange={(v) => handleToggleCoverage(task, v)}
                    />
                  </div>

                  <Space.Compact>
                    <Upload
                      beforeUpload={beforeUploadOverwrite(task.id)}
                      showUploadList={false}
                      accept=".zip,application/zip,application/x-zip-compressed"
                    >
                      <Button
                        icon={<UploadOutlined />}
                        size="small"
                        loading={uploadingFor === task.id}
                        style={{ height: 32 }}
                      >
                        Overwrite
                      </Button>
                    </Upload>

                    {isEditing ? (
                      <>
                        <Button
                          icon={<SaveOutlined />}
                          type="primary"
                          size="small"
                          onClick={() => handleSaveTask(task.id)}
                          loading={isSaving}
                          disabled={isSaving}
                          style={{ height: 32 }}
                        >
                          Save
                        </Button>
                        <Button size="small" onClick={cancelEdit} style={{ height: 32 }}>
                          Cancel
                        </Button>
                      </>
                    ) : (
                      <Button
                        icon={<EditOutlined />}
                        size="small"
                        onClick={() => beginEdit(task)}
                        style={{ height: 32 }}
                      >
                        Edit
                      </Button>
                    )}
                  </Space.Compact>

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
            );
          })}

          {/* Full-width Add Task button, same spacing as rows */}
          <Button
            icon={<PlusOutlined />}
            type="dashed"
            block
            onClick={handleCreateTask}
            data-cy="add-task"
            className="!h-[56px]"
          >
            Add Task
          </Button>
        </div>
      )}
    </div>
  );
};

export default StepTasks;
