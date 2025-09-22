import React from 'react';
import { Button, Input, Switch, Skeleton } from 'antd';
import { SaveOutlined } from '@ant-design/icons';
import SettingsGroup from '@/components/SettingsGroup';
import { useTasksPage } from './context';
import OverwriteFilesSection from './sections/OverwriteFilesSection';
import AssessmentSection from './sections/AssessmentSection';

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
    editedName,
    setEditedName,
    editedCommand,
    setEditedCommand,
    editedCoverage,
    setEditedCoverage,
    saveTask,
  } = useTasksPage();

  if (loading) return <TaskSkeleton />;

  // if (!selectedTask) {
  //   return (
  //     <Empty
  //       image={Empty.PRESENTED_IMAGE_SIMPLE}
  //       description={<div className="text-gray-700 dark:text-gray-300">No Task Selected</div>}
  //     />
  //   );
  // }

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

          <div className="flex justify-end">
            <Button icon={<SaveOutlined />} type="primary" onClick={saveTask}>
              Save Task
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
