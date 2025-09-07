import { useRef, useEffect } from 'react';
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

const AssignmentGrades = () => {
  const { setValue } = useViewSlot();
  const { isMobile } = useUI();
  const navigate = useNavigate();
  const moduleDetails = useModule();
  const { assignment } = useAssignment();
  const listRef = useRef<EntityListHandle>(null);

  const moduleId = moduleDetails.id;
  const assignmentId = assignment.id;

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
    sort: { field: string; order: 'ascend' | 'descend' }[];
  }): Promise<{ items: GradeResponse[]; total: number }> => {
    const res = await listGrades(moduleId, assignmentId, {
      page,
      per_page,
      query,
      sort,
    });

    if (res.success) {
      return { items: res.data.grades, total: res.data.total };
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
      message.error(e.message || 'Failed to export grades');
    }
  };

  const actions: EntityListProps<GradeResponse>['actions'] = {
    control: [
      {
        key: 'export',
        label: 'Export CSV',
        icon: <DownloadOutlined />,
        isPrimary: true,
        handler: handleExport,
      },
    ],
  };

  return (
    <div className="flex h-full flex-col gap-4">
      <EntityList<GradeResponse>
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
        columns={[
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
            title: 'Grade',
            dataIndex: 'score',
            key: 'score',
            align: 'right',
            sorter: { multiple: 2 },
            render: (_, g) => <PercentageTag value={g.score} decimals={1} palette="redGreen" />,
          },
          {
            title: 'Created At',
            dataIndex: 'created_at',
            key: 'created_at',
            defaultHidden: true,
            sorter: { multiple: 3 },
            render: (_, g) => <DateTime value={g.created_at} variant="datetime" />,
          },
          {
            title: 'Updated At',
            dataIndex: 'updated_at',
            key: 'updated_at',
            render: (_, g) => <DateTime value={g.updated_at} variant="datetime" />,
          },
        ]}
        emptyNoEntities={
          <AssignmentGradesEmptyState onRefresh={() => listRef.current?.refresh()} />
        }
      />
    </div>
  );
};

export default AssignmentGrades;
