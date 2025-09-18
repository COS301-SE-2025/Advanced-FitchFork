import { Button } from 'antd';
import { CheckCircleOutlined, CloseCircleOutlined } from '@ant-design/icons';

import type { AssignmentReadiness } from '@/types/modules/assignments';

type ChecklistAction =
  | { type: 'navigate'; href: string; label: string }
  | { type: 'generateMemo' }
  | { type: 'generateMark' };

type ChecklistItem = {
  key: keyof AssignmentReadiness | 'interpreter_present';
  label: string;
  show: boolean;
  action?: ChecklistAction;
};

type Props = {
  readiness: AssignmentReadiness | null;
  basePath: string;
  loading: boolean;
  shouldOfferMemoAction: boolean;
  shouldOfferMarkAction: boolean;
  onGenerateMemo: () => void | Promise<void>;
  onGenerateMark: () => void | Promise<void>;
  onNavigate: (path: string) => void;
};

const SetupChecklist = ({
  readiness,
  basePath,
  loading,
  shouldOfferMemoAction,
  shouldOfferMarkAction,
  onGenerateMemo,
  onGenerateMark,
  onNavigate,
}: Props) => {
  const submissionMode = readiness?.submission_mode;

  const items: ChecklistItem[] = [
    {
      key: 'config_present',
      label: 'Configuration file',
      show: true,
      action: { type: 'navigate', href: `${basePath}/config`, label: 'Open config' },
    },
    {
      key: 'main_present',
      label: 'Main file',
      show: submissionMode !== 'gatlam',
      action: {
        type: 'navigate',
        href: `${basePath}/config/files/main`,
        label: 'Upload main',
      },
    },
    {
      key: 'interpreter_present',
      label: 'Interpreter',
      show: submissionMode === 'gatlam',
      action: {
        type: 'navigate',
        href: `${basePath}/config/interpreter`,
        label: 'Configure interpreter',
      },
    },
    {
      key: 'makefile_present',
      label: 'Makefile',
      show: true,
      action: {
        type: 'navigate',
        href: `${basePath}/config/files/makefile`,
        label: 'Upload makefile',
      },
    },
    {
      key: 'memo_present',
      label: 'Memo file',
      show: true,
      action: {
        type: 'navigate',
        href: `${basePath}/config/files/memo`,
        label: 'Upload memo',
      },
    },
    {
      key: 'tasks_present',
      label: 'Tasks',
      show: true,
      action: {
        type: 'navigate',
        href: `${basePath}/tasks`,
        label: 'Manage tasks',
      },
    },
    {
      key: 'memo_output_present',
      label: 'Memo Output',
      show: true,
      action: { type: 'generateMemo' },
    },
    {
      key: 'mark_allocator_present',
      label: 'Mark Allocator',
      show: true,
      action: { type: 'generateMark' },
    },
  ];

  const renderAction = (item: ChecklistItem, complete: boolean) => {
    if (complete) {
      return (
        <span className="inline-flex w-full sm:w-auto justify-center items-center gap-1 text-xs font-medium px-2 py-1 rounded-full bg-green-100 text-green-700 dark:bg-green-900 dark:text-green-300">
          <CheckCircleOutlined /> Complete
        </span>
      );
    }

    const action = item.action;

    if (!action) {
      return (
        <span className="inline-flex w-full sm:w-auto justify-center items-center gap-1 text-xs font-medium px-2 py-1 rounded-full bg-red-100 text-red-700 dark:bg-red-900 dark:text-red-300">
          <CloseCircleOutlined /> Incomplete
        </span>
      );
    }

    switch (action.type) {
      case 'navigate':
        return (
          <Button
            size="small"
            className="!w-full sm:!w-auto"
            onClick={() => onNavigate(action.href)}
          >
            {action.label}
          </Button>
        );
      case 'generateMemo':
        return shouldOfferMemoAction ? (
          <Button
            size="small"
            type="primary"
            className="!w-full sm:!w-auto"
            onClick={() => void onGenerateMemo()}
            loading={loading}
            disabled={loading}
          >
            Generate memo output
          </Button>
        ) : (
          <span className="inline-flex w-full sm:w-auto justify-center items-center gap-1 text-xs font-medium px-2 py-1 rounded-full bg-yellow-100 text-yellow-700 dark:bg-yellow-900 dark:text-yellow-200">
            <CloseCircleOutlined /> Needs setup
          </span>
        );
      case 'generateMark':
        return shouldOfferMarkAction ? (
          <Button
            size="small"
            type="primary"
            className="!w-full sm:!w-auto"
            onClick={() => void onGenerateMark()}
            loading={loading}
            disabled={loading}
          >
            Generate mark allocator
          </Button>
        ) : (
          <span className="inline-flex w-full sm:w-auto justify-center items-center gap-1 text-xs font-medium px-2 py-1 rounded-full bg-yellow-100 text-yellow-700 dark:bg-yellow-900 dark:text-yellow-200">
            <CloseCircleOutlined /> Memo required
          </span>
        );
      default:
        return null;
    }
  };

  const visibleItems = items.filter((item) => item.show);

  return (
    <div className="flex flex-col gap-3 h-full">
      {visibleItems.map((item, index) => {
        const complete =
          item.key === 'interpreter_present'
            ? readiness?.interpreter_present
            : readiness?.[item.key as keyof AssignmentReadiness];

        return (
          <div
            key={item.key}
            className="rounded-lg border border-gray-200 dark:border-gray-700 bg-gray-50 dark:bg-gray-900 px-4 py-3"
          >
            <div className="flex flex-col gap-2 sm:flex-row sm:items-center sm:justify-between">
              <div className="flex items-center gap-3">
                <span className="flex h-7 w-7 items-center justify-center rounded-full bg-gray-200 text-xs font-semibold text-gray-700 dark:bg-gray-800 dark:text-gray-200">
                  {index + 1}
                </span>
                <span className="text-sm font-medium text-gray-800 dark:text-gray-200 leading-5">
                  {item.label}
                </span>
              </div>
              <div className="flex w-full sm:w-auto justify-start sm:justify-end">
                {renderAction(item, Boolean(complete))}
              </div>
            </div>
          </div>
        );
      })}
    </div>
  );
};

export default SetupChecklist;
