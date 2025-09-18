import React, { useEffect } from 'react';
import { Outlet } from 'react-router-dom';
import { useViewSlot } from '@/context/ViewSlotContext';
import { TasksPageProvider, useTasksPage } from './context';
import TaskList from './TaskList';
import { TasksEmptyState } from '@/components/tasks';
import { Typography } from 'antd';

const DesktopLayout: React.FC = () => {
  const {
    loading,
    tasks,
    createNewTask,
    hasMakefile,
    generateTasksFromMakefile,
    generatingFromMakefile,
  } = useTasksPage();
  if (loading) {
    return (
      <div className="flex-1 flex items-center justify-center text-gray-400 dark:text-gray-500">
        Loading tasks...
      </div>
    );
  }

  if (tasks.length === 0) {
    return (
      <TasksEmptyState
        onCreate={createNewTask}
        onGenerateFromMakefile={generateTasksFromMakefile}
        canGenerateFromMakefile={hasMakefile}
        generatingFromMakefile={generatingFromMakefile}
      />
    );
  }

  return (
    <div className="bg-white dark:bg-gray-900 border rounded-md border-gray-200 dark:border-gray-800 flex">
      <TaskList />
      <div className="flex-1 min-w-0 overflow-y-auto p-6">
        <div className="max-w-6xl">
          <Outlet />
        </div>
      </div>
    </div>
  );
};

const MobileLayout: React.FC = () => <Outlet />;

const TasksPageBody: React.FC = () => {
  const { isMobile } = useTasksPage();
  return isMobile ? <MobileLayout /> : <DesktopLayout />;
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
      <div className="flex-1 flex flex-col h-full">
        <TasksPageBody />
      </div>
    </TasksPageProvider>
  );
};

export default TasksPage;
