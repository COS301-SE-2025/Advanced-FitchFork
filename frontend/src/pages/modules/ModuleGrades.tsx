// TODO: Integrate this page with real API endpoints for grades and assignments instead of mock data.
//       Currently uses mock values for students, assignments, and grades.

import { useState, useRef } from 'react';
import { Tabs, InputNumber, Tag } from 'antd';
import { EditOutlined, DeleteOutlined, DownloadOutlined } from '@ant-design/icons';
import { EntityList, type EntityListHandle } from '@/components/EntityList';
import PageHeader from '@/components/PageHeader';
import { message } from '@/utils/message';
import { useModule } from '@/context/ModuleContext';
import { useAuth } from '@/context/AuthContext';
import type { ColumnsType } from 'antd/es/table';
import StatCard from '@/components/StatCard';

type GradeRecord = {
  id: number;
  studentNumber: string;
  grades: Record<string, number>;
  finalGrade: number;
};

type AssignmentRecord = {
  id: string;
  name: string;
  weight: number;
};

const ModuleGrades = () => {
  const module = useModule();
  const auth = useAuth();

  const isStudent = auth.isStudent(module.id);

  const gradeListRef = useRef<EntityListHandle>(null);
  const assignmentListRef = useRef<EntityListHandle>(null);

  const [activeTab, setActiveTab] = useState<'grades' | 'assignments'>('grades');
  const [assignments, setAssignments] = useState<AssignmentRecord[]>(
    Array.from({ length: 10 }, (_, i) => ({
      id: `a${i + 1}`,
      name: `Assignment ${i + 1}`,
      weight: +(1 / 10).toFixed(2),
    })),
  );

  const totalStudents = 42;
  const avgGrade = 68;
  const passingRate = 75;
  const assignmentsCount = 10;

  const ownFinalGrade = 72; // mock value for student's final grade
  const completedAssignments = 9;

  const generateStudentNumber = (): string => {
    let number = 'u';
    for (let i = 0; i < 8; i++) number += Math.floor(Math.random() * 10);
    return number;
  };

  const fetchGrades = async (): Promise<{ items: GradeRecord[]; total: number }> => {
    const items: GradeRecord[] = Array.from({ length: 10 }, (_, i) => {
      const grades: Record<string, number> = {};
      let weighted = 0;
      assignments.forEach((a) => {
        const mark = Math.floor(Math.random() * 81) + 20;
        grades[a.id] = mark;
        weighted += mark * a.weight;
      });
      return {
        id: i + 1,
        studentNumber: generateStudentNumber(),
        grades,
        finalGrade: Math.round(weighted),
      };
    });

    // if student, filter to own
    if (isStudent && auth.user) {
      const own = items.find((r) => r.studentNumber === auth.user?.username);
      return { items: own ? [own] : [], total: own ? 1 : 0 };
    }

    return { items, total: items.length };
  };

  const fetchAssignments = async (): Promise<{ items: AssignmentRecord[]; total: number }> => {
    return { items: assignments, total: assignments.length };
  };

  const gradeTagColor = (mark: number): string => {
    if (mark >= 75) return 'green';
    if (mark >= 50) return 'orange';
    return 'red';
  };

  const handleEditAssignment = async (values: Record<string, any>, record: AssignmentRecord) => {
    const updated = assignments.map((a) =>
      a.id === record.id ? { ...a, weight: values.weight } : a,
    );
    setAssignments(updated);
    message.success(`Updated weight of ${record.name}`);
    assignmentListRef.current?.refresh();
    gradeListRef.current?.refresh();
  };

  const handleEditStudent = ({ entity }: { entity?: GradeRecord }) => {
    if (entity) message.info(`Edit grades for ${entity.studentNumber}`);
  };

  const handleDeleteStudent = ({
    entity,
    refresh,
  }: {
    entity?: GradeRecord;
    refresh: () => void;
  }) => {
    if (entity) {
      message.success(`Deleted ${entity.studentNumber}`);
      refresh();
    }
  };

  const studentColumns: ColumnsType<GradeRecord> = [
    { title: 'Student Number', dataIndex: 'studentNumber', key: 'studentNumber' },
    ...assignments.map((a) => ({
      title: `${a.name} (${(a.weight * 100).toFixed(0)}%)`,
      dataIndex: ['grades', a.id],
      key: a.id,
      render: (mark: number) => <Tag color={gradeTagColor(mark)}>{mark}%</Tag>,
    })),
    {
      title: 'Final Grade',
      dataIndex: 'finalGrade',
      key: 'finalGrade',
      render: (mark: number) => <Tag color={gradeTagColor(mark)}>{mark}%</Tag>,
    },
  ];

  const studentSelfColumns: ColumnsType<AssignmentRecord> = [
    { title: 'Assignment', dataIndex: 'name', key: 'name' },
    {
      title: 'Weight',
      dataIndex: 'weight',
      key: 'weight',
      render: (weight: number) => `${(weight * 100).toFixed(0)}%`,
    },
    {
      title: 'Your Grade',
      dataIndex: 'id',
      key: 'grade',
      render: () => {
        const mark = Math.floor(Math.random() * 81) + 20; // mock grade
        return <Tag color={gradeTagColor(mark)}>{mark}%</Tag>;
      },
    },
  ];

  const assignmentColumns: ColumnsType<AssignmentRecord> = [
    { title: 'Name', dataIndex: 'name', key: 'name' },
    {
      title: 'Weight',
      dataIndex: 'weight',
      key: 'weight',
      render: (_, record) => (
        <InputNumber
          min={0}
          max={1}
          step={0.01}
          value={record.weight}
          onChange={(val) => {
            if (val != null) handleEditAssignment({ weight: val }, record);
          }}
        />
      ),
    },
  ];

  return (
    <div className="p-4 sm:p-6">
      <PageHeader title="Grades" description={`View and manage grades for ${module.code}`} />

      {/* Stat cards */}
      <div className="grid grid-cols-1 sm:grid-cols-2 md:grid-cols-4 gap-4 mb-6">
        {!isStudent && (
          <>
            <StatCard title="Total Students" value={totalStudents} />
            <StatCard title="Assignments" value={assignmentsCount} />
            <StatCard title="Average Grade" value={`${avgGrade}%`} />
            <StatCard title="Passing Rate" value={`${passingRate}%`} />
          </>
        )}

        {isStudent && (
          <>
            <StatCard title="Final Grade" value={`${ownFinalGrade}%`} />
            <StatCard title="Completed Assignments" value={completedAssignments} />
            <StatCard
              title="Remaining Assignments"
              value={assignmentsCount - completedAssignments}
            />
          </>
        )}
      </div>

      {/* Rest of the page */}
      {!isStudent && (
        <Tabs
          activeKey={activeTab}
          onChange={(key) => setActiveTab(key as 'grades' | 'assignments')}
          className="mb-4"
          items={[
            { key: 'grades', label: 'Grades' },
            { key: 'assignments', label: 'Assignments' },
          ]}
        />
      )}

      {!isStudent && activeTab === 'grades' && (
        <EntityList<GradeRecord>
          ref={gradeListRef}
          name="Grades"
          fetchItems={fetchGrades}
          getRowKey={(r) => r.id}
          columns={studentColumns}
          columnToggleEnabled
          actions={{
            control: [
              {
                key: 'mock-export-all',
                label: 'Export All to CSV',
                icon: <DownloadOutlined />,
                handler: ({ refresh }) => {
                  console.log('Mock export all grades to CSV');
                  message.success('Mock export: all grades would be exported');
                  refresh(); // Optional, only if you want to simulate refreshing
                },
              },
            ],
            entity: () => [
              {
                key: 'edit',
                label: 'Edit',
                icon: <EditOutlined />,
                handler: handleEditStudent,
              },
              {
                key: 'delete',
                label: 'Delete',
                icon: <DeleteOutlined />,
                confirm: true,
                handler: handleDeleteStudent,
              },
            ],
            bulk: [
              {
                key: 'mock-export-selected',
                label: 'Export Selected to CSV',
                icon: <DownloadOutlined />,
                handler: ({ selected }) => {
                  if (!selected || selected.length === 0) {
                    message.warning('No grades selected to export');
                    return;
                  }

                  console.log('Mock export selected grade IDs:', selected);
                  message.success(
                    `Mock export: ${selected.length} selected grades would be exported`,
                  );
                },
              },
            ],
          }}
        />
      )}

      {!isStudent && activeTab === 'assignments' && (
        <EntityList<AssignmentRecord>
          ref={assignmentListRef}
          name="Assignments"
          fetchItems={fetchAssignments}
          getRowKey={(a) => a.id}
          columns={assignmentColumns}
        />
      )}

      {isStudent && (
        <EntityList<AssignmentRecord>
          name="Your Grades"
          fetchItems={fetchAssignments}
          getRowKey={(a) => a.id}
          columns={studentSelfColumns}
        />
      )}
    </div>
  );
};

export default ModuleGrades;
