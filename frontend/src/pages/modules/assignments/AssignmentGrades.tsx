import { useRef, useEffect, useMemo, useState } from 'react';
import { useNavigate } from 'react-router-dom';
import type { EntityListHandle, EntityListProps } from '@/components/EntityList';
import { EntityList } from '@/components/EntityList';
import { message } from '@/utils/message';
import { Typography } from 'antd';
import { DownloadOutlined } from '@ant-design/icons';

import { useModule } from '@/context/ModuleContext';
import { useAssignment } from '@/context/AssignmentContext';
import { useViewSlot } from '@/context/ViewSlotContext';

import { listGrades, exportGrades } from '@/services/modules/assignments/grades';
import type { GradeResponse } from '@/services/modules/assignments/grades';

import { IdTag, DateTime, PercentageTag } from '@/components/common';
import AssignmentGradesEmptyState from '@/components/grades/AssignmentGradesEmptyState';
import { AssignmentGradeListItem } from '@/components/grades';
import { useUI } from '@/context/UIContext';
import type { SortOption } from '@/types/common';

// Helpers
const uniq = <T,>(xs: T[]) => Array.from(new Set(xs));
const taskIdFrom = (t: NonNullable<GradeResponse['tasks']>[number]) =>
  (t.name?.trim() || (t.task_number != null ? `Task ${t.task_number}` : 'Task')).replace(
    /\s+/g,
    ' ',
  );

const AssignmentGrades = () => {
  const { setValue } = useViewSlot();
  const { isMobile } = useUI();
  const navigate = useNavigate();
  const moduleDetails = useModule();
  const { assignment } = useAssignment();
  const listRef = useRef<EntityListHandle>(null);

  const moduleId = moduleDetails.id;
  const assignmentId = assignment.id;

  // Discovered task columns (union of task labels from the latest fetch)
  const [taskKeys, setTaskKeys] = useState<string[]>([]);

  // Set page title in slot
  useEffect(() => {
    setValue(
      <Typography.Text className="text-base font-medium text-gray-900 dark:text-gray-100 truncate">
        Grades
      </Typography.Text>,
    );
  }, [setValue]);

  // Fetch grades
  const fetchGrades = async ({
    page,
    per_page,
    query,
    sort,
  }: {
    page: number;
    per_page: number;
    query?: string;
    filters: Record<string, string[]>;
    sort: SortOption[];
  }): Promise<{ items: GradeResponse[]; total: number }> => {
    const res = await listGrades(moduleId, assignmentId, {
      page,
      per_page,
      query,
      sort,
    });

    if (res.success) {
      const items = res.data.grades ?? [];
      const discovered = items.flatMap((g) =>
        (g.tasks ?? []).map(taskIdFrom).filter((x): x is string => !!x),
      );
      if (discovered.length) setTaskKeys((prev) => uniq([...prev, ...discovered]));
      return { items, total: res.data.total };
    } else {
      message.error(`Failed to fetch grades: ${res.message}`);
      return { items: [], total: 0 };
    }
  };

  // Export as CSV
  const handleExport = async () => {
    try {
      await exportGrades(moduleId, assignmentId);
      message.success('Export started – check your downloads');
    } catch (e: any) {
      message.error(e?.message || 'Failed to export grades');
    }
  };

  const actions = useMemo<EntityListProps<GradeResponse>['actions']>(
    () => ({
      control: [
        {
          key: 'export',
          label: 'Export CSV',
          icon: <DownloadOutlined />,
          isPrimary: true,
          handler: handleExport,
        },
      ],
    }),
    [handleExport],
  );

  // Base columns (without Grade)
  const baseColumnsWithoutGrade: EntityListProps<GradeResponse>['columns'] = useMemo(
    () => [
      {
        title: 'ID',
        dataIndex: 'id',
        key: 'id',
        defaultHidden: true,
        render: (id: number) => <IdTag id={id} />,
      },
      {
        title: 'Username',
        dataIndex: 'username',
        key: 'username',
        sorter: { multiple: 1 },
      },
      {
        // renamed
        title: 'Submitted At',
        dataIndex: 'created_at',
        key: 'created_at',
        sorter: { multiple: 2 },
        render: (_: unknown, g: GradeResponse) => (
          <DateTime value={g.created_at} variant="datetime" />
        ),
      },
      {
        title: 'Updated At',
        dataIndex: 'updated_at',
        defaultHidden: true,
        key: 'updated_at',
        render: (_: unknown, g: GradeResponse) => (
          <DateTime value={g.updated_at} variant="datetime" />
        ),
      },
    ],
    [],
  );

  // Dynamic task columns (hidden by default)
  const dynamicTaskColumns: EntityListProps<GradeResponse>['columns'] = useMemo(
    () =>
      taskKeys.map((taskKey) => ({
        title: taskKey,
        key: `task:${taskKey}`,
        dataIndex: `task:${taskKey}`,
        align: 'right' as const,
        render: (_: unknown, g: GradeResponse) => {
          const t = (g.tasks || []).find((x) => taskIdFrom(x) === taskKey);
          return t ? <PercentageTag value={t.score} decimals={1} /> : '—';
        },
      })),
    [taskKeys],
  );

  // Grade column (moved to very end)
  const gradeColumn: EntityListProps<GradeResponse>['columns'][number] = useMemo(
    () => ({
      title: 'Grade',
      dataIndex: 'score',
      key: 'score',
      align: 'right',
      sorter: { multiple: 3 }, // any priority is fine; it’s last visually
      render: (_: unknown, g: GradeResponse) => (
        <PercentageTag value={g.score} decimals={1} scheme="red-green" />
      ),
    }),
    [],
  );

  // Final columns: base + tasks + grade LAST
  const allColumns = useMemo(
    () => [...baseColumnsWithoutGrade, ...dynamicTaskColumns, gradeColumn],
    [baseColumnsWithoutGrade, dynamicTaskColumns, gradeColumn],
  );

  return (
    <div className="flex h-full flex-col gap-4">
      <EntityList<GradeResponse>
        key={`grades:${taskKeys.join('|')}`} // remount when new task columns appear
        ref={listRef}
        name="Grades"
        fetchItems={fetchGrades}
        getRowKey={(g) => g.id}
        onRowClick={(g) =>
          navigate(`/modules/${moduleId}/assignments/${assignmentId}/grades/${g.id}`)
        }
        listMode={isMobile}
        renderListItem={(grade) => (
          <AssignmentGradeListItem
            grade={grade}
            onClick={(g) =>
              navigate(`/modules/${moduleId}/assignments/${assignmentId}/grades/${g.id}`)
            }
          />
        )}
        columnToggleEnabled
        actions={actions}
        columns={allColumns}
        emptyNoEntities={
          <AssignmentGradesEmptyState onRefresh={() => listRef.current?.refresh()} />
        }
        filtersStorageKey={`modules:${moduleId}:assignments:${assignmentId}:grades:filters:v1`}
      />
    </div>
  );
};

export default AssignmentGrades;
