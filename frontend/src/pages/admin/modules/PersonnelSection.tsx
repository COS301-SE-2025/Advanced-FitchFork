import { Table, Typography, Space, Tag, Button } from 'antd';
import { EditOutlined, DeleteOutlined } from '@ant-design/icons';
import type { ColumnsType } from 'antd/es/table';

const { Title } = Typography;

interface Person {
  key: string;
  name: string;
  email: string;
  role: 'Lecturer' | 'Tutor' | 'Student';
}

const personnelData: Person[] = [
  { key: '1', name: 'Dr. Alice van Rensburg', email: 'alice@up.ac.za', role: 'Lecturer' },
  { key: '2', name: 'Tom Mokoena', email: 'tom@up.ac.za', role: 'Tutor' },
  { key: '3', name: 'Lebo Ndlovu', email: 'lebo123@students.up.ac.za', role: 'Student' },
];

const personnelColumns: ColumnsType<Person> = [
  {
    title: 'Name',
    dataIndex: 'name',
    key: 'name',
    render: (text) => <span className="font-medium">{text}</span>,
  },
  { title: 'Email', dataIndex: 'email', key: 'email' },
  {
    title: 'Role',
    dataIndex: 'role',
    key: 'role',
    filters: ['Lecturer', 'Tutor', 'Student'].map((r) => ({ text: r, value: r })),
    onFilter: (value, record) => record.role === value,
    render: (role) => (
      <Tag color={role === 'Lecturer' ? 'volcano' : role === 'Tutor' ? 'geekblue' : 'green'}>
        {role}
      </Tag>
    ),
  },
  {
    title: 'Actions',
    key: 'actions',
    render: () => (
      <Space>
        <Button size="small" icon={<EditOutlined />} />
        <Button size="small" icon={<DeleteOutlined />} danger />
      </Space>
    ),
  },
];

export default function PersonnelSection() {
  return (
    <div>
      <Table columns={personnelColumns} dataSource={personnelData} pagination={false} bordered />
    </div>
  );
}
