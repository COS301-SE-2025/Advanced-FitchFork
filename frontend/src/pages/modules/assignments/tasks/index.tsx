import React, { useEffect } from 'react';
import { Button, Empty, Typography } from 'antd';
import { useViewSlot } from '@/context/ViewSlotContext';
import TaskView from './TaskView';
import { TasksPageProvider, useTasksPage } from './context';
import TaskList from './TaskList';

const DesktopLayout: React.FC = () => {
  const { loading, tasks, createNewTask } = useTasksPage();
  return (
    <div className="bg-white dark:bg-gray-900 border rounded-md border-gray-200 dark:border-gray-800 flex h-full min-h-0 overflow-hidden">
      <TaskList />
      <div className="flex-1 min-w-0 min-h-0 overflow-y-auto p-6">
        <div className="w-full max-w-6xl">
          {loading ? (
            <div className="text-gray-400">Loading tasks...</div>
          ) : tasks.length === 0 ? (
            <Empty
              description={<div className="text-gray-700 dark:text-gray-300">No Tasks Found</div>}
            >
              <Button type="primary" onClick={createNewTask}>
                + New Task
              </Button>
            </Empty>
          ) : (
            <TaskView />
          )}
        </div>
      </div>
    </div>
  );
};

const TasksPageBody: React.FC = () => {
  // (Optional) You can implement a Mobile layout later using the same context
  return <DesktopLayout />;
};

const TasksPage: React.FC = () => {
  const { setValue } = useViewSlot();

  useEffect(() => {
    setValue(
      <Typography.Text
        className="text-base font-medium text-gray-900 dark:text-gray-100 truncate"
        title="Tasks"
      >
        Tasks
      </Typography.Text>,
    );
  }, [setValue]);

  return (
    <TasksPageProvider>
      <TasksPageBody />
    </TasksPageProvider>
  );
};

export default TasksPage;
