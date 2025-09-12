import React from 'react';
import { Button, Empty, Input, Switch } from 'antd';
import { SaveOutlined } from '@ant-design/icons';
import SettingsGroup from '@/components/SettingsGroup';
import { useTasksPage } from './context';
import OverwriteFilesSection from './sections/OverwriteFilesSection';
import AssessmentSection from './sections/AssessmentSection';

const TaskView: React.FC = () => {
  const {
    loading,
    selectedTask,
    editedName,
    setEditedName,
    editedCommand,
    setEditedCommand,
    editedCoverage,
    setEditedCoverage,
    saveTask,
  } = useTasksPage();

  if (loading) return <div className="text-gray-400">Loading tasks...</div>;
  if (!selectedTask)
    return (
      <Empty
        description={<div className="text-gray-700 dark:text-gray-300">No Task Selected</div>}
      ></Empty>
    );

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
