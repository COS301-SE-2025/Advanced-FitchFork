// src/pages/modules/assignments/Tasks/TaskList.tsx
import React from 'react';
import { Button, Dropdown, Menu, Modal } from 'antd';
import { MoreOutlined } from '@ant-design/icons';
import { useTasksPage } from './context';

const TaskList: React.FC = () => {
  const { tasks, selectedId, setSelectedId, createNewTask, deleteTask } = useTasksPage();

  const items = tasks.map((task) => ({
    key: task.id.toString(),
    label: (
      <div className="flex items-center justify-between gap-2 min-w-0">
        <span className="truncate text-left">
          {task.name || `Task ${task.task_number}`}
        </span>
        <Dropdown
          trigger={['click']}
          menu={{
            items: [
              {
                key: 'delete',
                danger: true,
                label: (
                  <span
                    onClick={(e) => {
                      e.stopPropagation();
                      Modal.confirm({
                        title: 'Delete this task?',
                        content: 'This task and its configuration will be removed.',
                        okText: 'Delete',
                        okButtonProps: { danger: true },
                        cancelText: 'Cancel',
                        onOk: () => deleteTask(task.id),
                      });
                    }}
                  >
                    Delete
                  </span>
                ),
              },
            ],
          }}
        >
          <Button
            type="text"
            size="small"
            icon={<MoreOutlined />}
            onClick={(e) => e.stopPropagation()}
            className="flex-shrink-0"
          />
        </Dropdown>
      </div>
    ),
  }));

  return (
    <div className="w-[240px] bg-gray-50 dark:bg-gray-950 border-r border-gray-200 dark:border-gray-800 px-2 py-2 overflow-y-auto">
      <Menu
        mode="inline"
        theme="light"
        selectedKeys={selectedId ? [selectedId.toString()] : []}
        onClick={({ key }) => setSelectedId(Number(key))}
        items={items}
        className="!bg-transparent !p-0"
        style={{ border: 'none' }}
      />
      <div className="px-1 mt-4">
        <Button block type="dashed" onClick={createNewTask}>
          + New Task
        </Button>
      </div>
    </div>
  );
};

export default TaskList;
