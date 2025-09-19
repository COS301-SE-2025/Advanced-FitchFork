import React, { useEffect } from 'react';
import { Button, Input, Switch, Skeleton, Modal, Typography } from 'antd';
import { DeleteOutlined, SaveOutlined } from '@ant-design/icons';
import SettingsGroup from '@/components/SettingsGroup';
import { useTasksPage } from './context';
import OverwriteFilesSection from './sections/OverwriteFilesSection';
import AssessmentSection from './sections/AssessmentSection';
import { useViewSlot } from '@/context/ViewSlotContext';

const TaskSkeleton: React.FC = () => (
  <div className="space-y-6">
    <SettingsGroup title="Task" description="Basic info and execution command for this task.">
      <div className="space-y-6">
        {/* Task Name */}
        <div>
          <div className="mb-1">
            <Skeleton.Input active size="small" style={{ width: 120 }} />
          </div>
          <Skeleton.Input active style={{ width: '100%', height: 32 }} />
        </div>

        {/* Command */}
        <div>
          <div className="mb-1">
            <Skeleton.Input active size="small" style={{ width: 90 }} />
          </div>
          <Skeleton.Input active style={{ width: '100%', height: 32 }} />
        </div>

        {/* Code Coverage */}
        <div>
          <div className="mb-1">
            <Skeleton.Input active size="small" style={{ width: 130 }} />
          </div>
          {/* Switch stand-in */}
          <Skeleton.Button active style={{ width: 52, height: 24, borderRadius: 12 }} />
        </div>

        {/* Save button */}
        <div className="flex justify-end">
          <Skeleton.Button active style={{ width: 110, height: 32 }} />
        </div>
      </div>
    </SettingsGroup>

    {/* Sections below can use generic skeletons */}
    <Skeleton active title={{ width: 180 }} paragraph={{ rows: 4 }} />
    <Skeleton active title={{ width: 220 }} paragraph={{ rows: 6 }} />
  </div>
);

const TaskView: React.FC = () => {
  const {
    loading,
    moduleId,
    assignmentId,
    selectedTask,
    editedName,
    setEditedName,
    editedCommand,
    setEditedCommand,
    editedCoverage,
    setEditedCoverage,
    saveTask,
    deleteTask,
  } = useTasksPage();
  const { setValue, setBackTo } = useViewSlot();

  useEffect(() => {
    return () => {
      setBackTo(null);
      setValue(
        <Typography.Text
          className="text-base font-medium text-gray-900 dark:text-gray-100 truncate"
          title="Tasks"
        >
          Tasks
        </Typography.Text>,
      );
    };
  }, [setBackTo, setValue]);

  useEffect(() => {
    if (!selectedTask) return;
    const label = (selectedTask.name && selectedTask.name.trim())
      || `Task ${selectedTask.task_number ?? selectedTask.id}`;
    setValue(
      <Typography.Text
        className="text-base font-medium text-gray-900 dark:text-gray-100 truncate"
        title={label}
      >
        {label}
      </Typography.Text>,
    );
    setBackTo(`/modules/${moduleId}/assignments/${assignmentId}/tasks`);
  }, [selectedTask, setValue, setBackTo, moduleId, assignmentId]);

  const handleDelete = () => {
    if (!selectedTask) return;
    Modal.confirm({
      title: 'Delete this task?',
      content: 'This task and its configuration will be removed.',
      okText: 'Delete',
      okButtonProps: { danger: true },
      cancelText: 'Cancel',
      onOk: () => deleteTask(selectedTask.id),
    });
  };

  if (loading) return <TaskSkeleton />;

  if (!selectedTask) {
    return (
      <div className="text-gray-400 dark:text-gray-500">Select a task to view its details.</div>
    );
  }

  return (
    <div className="space-y-6">
      <SettingsGroup title="Task" description="Basic info and execution command for this task.">
        <div className="space-y-6">
          <div>
            <label className="block font-medium mb-1">Task Name</label>
            <Input
              value={editedName}
              onChange={(e) => setEditedName(e.target.value)}
              className="w-full"
            />
          </div>

          <div>
            <label className="block font-medium mb-1">Command</label>
            <Input
              value={editedCommand}
              onChange={(e) => setEditedCommand(e.target.value)}
              className="w-full"
            />
          </div>

          <div>
            <label className="block font-medium mb-1">Code Coverage</label>
            <div>
              <Switch checked={editedCoverage} onChange={(checked) => setEditedCoverage(checked)} />
            </div>
          </div>

          <div className="flex flex-col sm:flex-row gap-2 justify-end">
            <Button
              icon={<SaveOutlined />}
              type="primary"
              onClick={saveTask}
              className="w-full sm:w-auto"
            >
              Save Task
            </Button>
            <Button
              danger
              icon={<DeleteOutlined />}
              onClick={handleDelete}
              className="w-full sm:w-auto"
            >
              Delete Task
            </Button>
          </div>
        </div>
      </SettingsGroup>

      {/* Overwrite Files */}
      <OverwriteFilesSection />

      {/* Assessment */}
      <AssessmentSection />
    </div>
  );
};

export default TaskView;
