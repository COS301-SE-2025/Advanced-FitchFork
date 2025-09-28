// src/pages/modules/assignments/setup/steps/StepTasks.tsx
import { useEffect, useMemo, useState } from 'react';
import {
  Typography,
  Button,
  Input,
  Spin,
  Empty,
  message,
  Upload,
  Tooltip,
  Space,
  Segmented,
} from 'antd';
import {
  PlusOutlined,
  SaveOutlined,
  EditOutlined,
  DeleteOutlined,
  UploadOutlined,
  ThunderboltOutlined,
} from '@ant-design/icons';
import Tip from '@/components/common/Tip';

import { useModule } from '@/context/ModuleContext';
import { useAssignmentSetup } from '@/context/AssignmentSetupContext';
import { listTasks, createTask, editTask, deleteTask } from '@/services/modules/assignments/tasks';
import { uploadOverwriteFiles } from '@/services/modules/assignments/overwrite_files/post';
import { fetchAssignmentFileBlob } from '@/services/modules/assignments';
import {
  parseTargetsFromMakefileZip,
  createTasksFromMakefileTargets,
} from '@/utils/makefile_tasks';

import type { TaskType } from '@/types/modules/assignments/tasks';
import { taskTypeOptionsForLanguage } from '@/policies/languages';

const { Title, Paragraph, Text } = Typography;

type TaskRow = {
  id: number;
  task_number: number;
  name: string;
  command: string;
  task_type: TaskType;
};

const StepTasks = () => {
  const module = useModule();
  const { assignmentId, assignment, readiness, refreshAssignment, setStepSaveHandler, config } =
    useAssignmentSetup();

  const lang = config?.project.language ?? null;

  const typeOptions = useMemo(() => taskTypeOptionsForLanguage(lang), [lang]);
  const allowedValues = useMemo(() => new Set(typeOptions.map((o) => o.value)), [typeOptions]);

  const [tasks, setTasks] = useState<TaskRow[]>([]);
  const [loading, setLoading] = useState(false);

  const [editingTaskId, setEditingTaskId] = useState<number | null>(null);
  const [editedName, setEditedName] = useState('');
  const [editedCommand, setEditedCommand] = useState('');
  const [editedType, setEditedType] = useState<TaskType>('normal');

  const [savingId, setSavingId] = useState<number | null>(null);
  const [uploadingFor, setUploadingFor] = useState<number | null>(null);
  const [generatingFromMakefile, setGeneratingFromMakefile] = useState(false);

  const fetchTasks = async () => {
    if (!assignmentId) return;
    setLoading(true);
    try {
      const res = await listTasks(module.id, assignmentId);
      if (res.success) {
        const data = (res.data as TaskRow[]).sort((a, b) => a.task_number - b.task_number);
        setTasks(data);
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

  const makefileFile = assignment?.files?.find((f) => f.file_type === 'makefile') ?? null;
  const hasMakefile = !!(readiness?.makefile_present && makefileFile);

  const handleGenerateFromMakefile = async () => {
    if (!assignmentId || !makefileFile) {
      message.info('Upload a Makefile in the previous step before generating tasks.');
      return;
    }

    if (generatingFromMakefile) return;

    setGeneratingFromMakefile(true);
    try {
      const blob = await fetchAssignmentFileBlob(module.id, assignmentId, makefileFile.id);
      const file = new File([blob], makefileFile.filename, {
        type: blob.type || 'application/zip',
      });

      const targets = await parseTargetsFromMakefileZip(file);
      if (!targets.length) {
        message.info('No runnable targets were detected in the Makefile.');
        return;
      }

      const created = await createTasksFromMakefileTargets(
        module.id,
        assignmentId,
        targets,
        refreshAssignment,
      );

      if (created > 0) {
        await fetchTasks();
        message.success(`Generated ${created} task${created === 1 ? '' : 's'} from the Makefile.`);
      } else {
        message.info('No new tasks were created from the Makefile.');
      }
    } catch (err) {
      message.error('Failed to generate tasks from the Makefile.');
      // eslint-disable-next-line no-console
      console.error(err);
    } finally {
      setGeneratingFromMakefile(false);
    }
  };

  const coerceAllowed = (t: TaskType): TaskType => (allowedValues.has(t) ? t : 'normal');

  const handleCreateTask = async () => {
    if (!assignmentId) return;

    const nextTaskNumber = (tasks[tasks.length - 1]?.task_number ?? 0) + 1;

    const res = await createTask(module.id, assignmentId, {
      task_number: nextTaskNumber,
      name: `Task ${nextTaskNumber}`,
      command: 'echo Hello World',
      task_type: 'normal',
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
    if (savingId === taskId) return;

    const original = tasks.find((t) => t.id === taskId);
    if (!original) return;

    const nextType = coerceAllowed(editedType);

    if (
      editedName.trim() === original.name &&
      editedCommand.trim() === original.command &&
      nextType === original.task_type
    ) {
      setEditingTaskId(null);
      return;
    }

    try {
      setSavingId(taskId);
      const res = await editTask(module.id, assignmentId, taskId, {
        name: editedName.trim(),
        command: editedCommand.trim(),
        task_type: nextType,
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
    return false;
  };

  const beginEdit = (task: TaskRow) => {
    setEditingTaskId(task.id);
    setEditedName(task.name);
    setEditedCommand(task.command);
    setEditedType(coerceAllowed(task.task_type));
  };

  const cancelEdit = () => {
    setEditingTaskId(null);
    setEditedName('');
    setEditedCommand('');
    setEditedType('normal');
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

      {tasks.length > 0 && (
        <div className="flex flex-col sm:flex-row sm:items-center gap-2">
          <Button
            icon={<ThunderboltOutlined />}
            type="default"
            onClick={() => void handleGenerateFromMakefile()}
            disabled={!hasMakefile || generatingFromMakefile}
            loading={generatingFromMakefile}
          >
            Generate tasks from Makefile
          </Button>
          {!hasMakefile && (
            <Text type="secondary" className="text-xs">
              Upload a Makefile in the Files & Resources step to enable automatic task generation.
            </Text>
          )}
        </div>
      )}

      {loading ? (
        <Spin />
      ) : tasks.length === 0 ? (
        <div className="rounded-2xl border border-dashed border-gray-300 dark:border-gray-700 bg-white dark:bg-gray-950 p-8 text-center space-y-6">
          <Empty
            description={
              <div className="space-y-1">
                <Text className="text-base font-medium text-gray-900 dark:text-gray-100">
                  No tasks yet
                </Text>
                <Paragraph className="!m-0 !text-sm !text-gray-600 dark:!text-gray-300">
                  Add tasks manually or generate them from your uploaded Makefile.
                </Paragraph>
              </div>
            }
            image={Empty.PRESENTED_IMAGE_SIMPLE}
          />

          <Space wrap size="middle" className="w-full justify-center">
            <Button
              type="primary"
              icon={<PlusOutlined />}
              onClick={handleCreateTask}
              data-cy="add-task"
              size="large"
            >
              Add Task
            </Button>
            <Button
              icon={<ThunderboltOutlined />}
              onClick={() => void handleGenerateFromMakefile()}
              disabled={!hasMakefile || generatingFromMakefile}
              loading={generatingFromMakefile}
              size="large"
            >
              Generate from Makefile
            </Button>
          </Space>

          {!hasMakefile && (
            <Text type="secondary" className="block text-xs text-gray-500 dark:text-gray-400">
              Tip: upload a Makefile in the Files & Resources step to unlock automatic task
              creation.
            </Text>
          )}
        </div>
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
                  {/* Task Type */}
                  <div className="flex items-center gap-2">
                    <Tooltip title="How this task is executed/assessed">
                      <span className="text-xs text-gray-600 dark:text-gray-300">Mode</span>
                    </Tooltip>
                    {isEditing ? (
                      <Segmented
                        value={editedType}
                        onChange={(v) => setEditedType(v as TaskType)}
                        options={typeOptions}
                        size="small"
                      />
                    ) : (
                      <Segmented
                        value={task.task_type}
                        options={typeOptions}
                        size="small"
                        disabled
                      />
                    )}
                  </div>

                  <div className="flex items-center gap-2">
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
                    <Tip
                      iconOnly
                      newTab
                      to="/help/assignments/tasks#overwrite"
                      text="Overwrite files help"
                    />
                  </div>

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
