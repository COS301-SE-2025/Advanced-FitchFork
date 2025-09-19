import { useEffect, useMemo, useState } from 'react';
import { Button, Typography } from 'antd';
import { ProfileOutlined, RightOutlined, PlusOutlined } from '@ant-design/icons';
import { useAssignment } from '@/context/AssignmentContext';
import { useModule } from '@/context/ModuleContext';
import { useTasksPage } from './context';
import { useViewSlot } from '@/context/ViewSlotContext';
import { TasksEmptyState } from '@/components/tasks';

const TasksMobileMenu = () => {
  const { assignment } = useAssignment();
  const module = useModule();
  const {
    tasks,
    loading,
    setSelectedId,
    createNewTask,
    hasMakefile,
    generateTasksFromMakefile,
    generatingFromMakefile,
  } = useTasksPage();
  const [creating, setCreating] = useState(false);
  const { setBackTo } = useViewSlot();

  useEffect(() => {
    setBackTo(null);
  }, [setBackTo]);

  const headerSubtitle = useMemo(() => {
    const pieces = [module.code];
    if (module.year) {
      pieces.push(module.year.toString());
    }
    return pieces.filter(Boolean).join(' • ');
  }, [module.code, module.year]);

  const handleSelectTask = (taskId: number) => {
    setSelectedId(taskId);
  };

  const handleCreateTask = async () => {
    try {
      setCreating(true);
      await createNewTask();
    } finally {
      setCreating(false);
    }
  };

  if (loading) {
    return (
      <div className="flex-1 flex items-center justify-center text-sm text-gray-400 dark:text-gray-500">
        Loading tasks…
      </div>
    );
  }

  if (tasks.length === 0) {
    return (
      <TasksEmptyState
        onCreate={handleCreateTask}
        loading={creating}
        onGenerateFromMakefile={generateTasksFromMakefile}
        canGenerateFromMakefile={hasMakefile}
        generatingFromMakefile={generatingFromMakefile}
      />
    );
  }

  return (
    <div className="h-full flex flex-col overflow-hidden">
      <div className="flex-1 overflow-y-auto space-y-6">
        <div className="rounded-lg border border-gray-200 dark:border-gray-800 p-4 bg-white dark:bg-neutral-900">
          <Typography.Title level={5} className="!m-0">
            {assignment?.name ?? 'Tasks'}
          </Typography.Title>
          {headerSubtitle ? (
            <Typography.Text type="secondary">{headerSubtitle}</Typography.Text>
          ) : null}
        </div>

        <div>
          <div className="flex items-center justify-between mb-2">
            <Typography.Text className="text-sm font-medium text-gray-500 dark:text-gray-400">
              Tasks
            </Typography.Text>
            <Typography.Text type="secondary">{tasks.length} total</Typography.Text>
          </div>

          <div className="!space-y-2">
            {tasks.map((task) => {
              const label = task.name || `Task ${task.task_number}`;
              return (
                <button
                  key={task.id}
                  type="button"
                  onClick={() => handleSelectTask(task.id)}
                  className="w-full px-4 py-3 border border-gray-200 dark:border-gray-800 rounded-lg bg-white dark:bg-gray-950 flex items-center justify-between gap-3 active:bg-gray-100 dark:active:bg-gray-900"
                >
                  <span className="flex items-center gap-2 text-left text-gray-900 dark:text-gray-100 min-w-0">
                    <ProfileOutlined className="text-lg flex-shrink-0" />
                    <span className="truncate">{label}</span>
                  </span>
                  <RightOutlined className="text-gray-400 flex-shrink-0" />
                </button>
              );
            })}
          </div>
        </div>
      </div>

      <div className="pt-4">
        <Button
          type="primary"
          block
          icon={<PlusOutlined />}
          onClick={handleCreateTask}
          loading={creating}
          className="h-12"
        >
          New Task
        </Button>
      </div>
    </div>
  );
};

export default TasksMobileMenu;
