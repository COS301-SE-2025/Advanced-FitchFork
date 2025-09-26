import { useMemo } from 'react';
import { Alert, Descriptions, List, Space, Table, Tag, Typography } from 'antd';
import type { ColumnsType } from 'antd/es/table';
import { useHelpToc } from '@/context/HelpContext';
import { useHelpPageHeader } from '@/pages/help/useHelpPageHeader';

const { Title, Paragraph, Text } = Typography;

const toc = [
  { key: 'overview', href: '#overview', title: 'Module roles at a glance' },
  { key: 'roles', href: '#roles', title: 'Role capabilities' },
  { key: 'assign', href: '#assign', title: 'Assign or remove people' },
  { key: 'filters', href: '#filters', title: 'Search & filters' },
  { key: 'best-practices', href: '#best-practices', title: 'Best practices' },
];

const roleData = [
  {
    key: 'lecturer',
    role: 'Lecturer',
    powers:
      'Full read/write access to the module, assignments, attendance, personnel management, and settings.',
  },
  {
    key: 'assistant',
    role: 'Assistant Lecturer',
    powers:
      'Manage assignments, mark allocation, attendance sessions, and grade reviews without access to module settings.',
  },
  {
    key: 'tutor',
    role: 'Tutor',
    powers: 'Access submissions, memo output, and attendance history. Cannot edit configuration.',
  },
  {
    key: 'student',
    role: 'Student',
    powers: 'Submit work, view feedback, check attendance, and read announcements.',
  },
];

const roleColumns: ColumnsType<(typeof roleData)[number]> = [
  {
    dataIndex: 'role',
    title: 'Role',
    width: 160,
    render: (value) => <Tag color="blue">{value}</Tag>,
  },
  {
    dataIndex: 'powers',
    title: 'What they can do',
  },
];

export default function ModulePersonnelHelp() {
  useHelpPageHeader('modules/personnel', { label: 'Personnel & Roles' });
  const ids = useMemo(() => toc.map((t) => t.href.slice(1)), []);

  useHelpToc({ items: toc, ids, onMountScrollToHash: true });

  return (
    <Space direction="vertical" size="large" className="w-full !p-0">
      <div>
        <Title level={2} className="mb-2">
          Personnel &amp; Roles
        </Title>
        <Paragraph className="mb-0 text-gray-600 dark:text-gray-300">
          Keep module membership tidy by assigning users to lecturer, assistant, tutor, or student
          roles. The Personnel tool uses a transfer list so you can bulk assign or remove users from
          a role without losing context.
        </Paragraph>
      </div>

      <section id="overview" className="scroll-mt-32">
        <Title level={3}>Module roles at a glance</Title>
        <Paragraph>
          Open <Text strong>Module → Personnel</Text> to see two panes: eligible users on the left and
          people already in the selected role on the right. The role selector above the table lets
          you switch between lecturers, assistants, tutors, and students without leaving the page.
        </Paragraph>
      </section>

      <section id="roles" className="scroll-mt-32">
        <Title level={3}>Role capabilities</Title>
        <Table
          size="small"
          columns={roleColumns}
          dataSource={roleData}
          pagination={false}
          rowKey="key"
          className="hidden md:block"
        />
        <List
          size="small"
          className="md:hidden"
          dataSource={roleData}
          renderItem={(item) => (
            <List.Item className="flex-col items-start">
              <Tag color="blue" className="mb-1">
                {item.role}
              </Tag>
              <span className="text-gray-700 dark:text-gray-200">{item.powers}</span>
            </List.Item>
          )}
        />
      </section>

      <section id="assign" className="scroll-mt-32">
        <Title level={3}>Assign or remove people</Title>
        <List
          bordered
          size="small"
          dataSource={[
            'Choose a role in the segmented control (Lecturer, Assistant Lecturer, Tutor, Student).',
            'Use the filters or search to find eligible users. Results respect pagination and search terms.',
            'Select people in the left table and click the transfer arrow to move them into the role.',
            'The right table updates immediately. Removing someone uses the same transfer control in reverse.',
            'FitchFork logs each change so audit history reflects who assigned or removed access.',
          ]}
          renderItem={(item) => <List.Item className="text-gray-700 dark:text-gray-200">{item}</List.Item>}
        />
        <Alert
          className="mt-4"
          type="info"
          showIcon
          message="Lecturer access"
          description="Only platform admins can add or remove lecturers. Assistant lecturers can manage every other role."
        />
      </section>

      <section id="filters" className="scroll-mt-32">
        <Title level={3}>Search &amp; filters</Title>
        <Descriptions bordered size="middle" column={1} className="mt-3">
          <Descriptions.Item label="Search">
            Use the search fields above each table to filter by username or email. Filters apply to
            API requests so you work with the real dataset, not just the current page.
          </Descriptions.Item>
          <Descriptions.Item label="Pagination">
            Every transfer pane keeps its own pagination and page size. FitchFork remembers your
            choice while you manage roles so you can jump between pages without losing context.
          </Descriptions.Item>
        </Descriptions>
      </section>

      <section id="best-practices" className="scroll-mt-32">
        <Title level={3}>Best practices</Title>
        <List
          bordered
          size="small"
          dataSource={[
            'Leave at least two lecturers in a module so ownership is never blocked.',
            'Use assistant lecturers for senior tutors who need assignment configuration access.',
            'Keep tutors limited to marking roles; they do not see module settings or mark allocation updates.',
            'Remove graduated students from the Personnel list to revoke access to submissions and memo output.',
          ]}
          renderItem={(item) => <List.Item className="text-gray-700 dark:text-gray-200">{item}</List.Item>}
        />
      </section>
    </Space>
  );
}
