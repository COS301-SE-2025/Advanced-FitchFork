import { Table, Form, Input, Select, Button, Tag, Space } from 'antd';
import { EditOutlined, SaveOutlined, CloseOutlined, DeleteOutlined } from '@ant-design/icons';
import { useState } from 'react';
import DashboardLayout from '@layouts/DashboardLayout';
import { useNavigate } from 'react-router-dom';

const { Option } = Select;

const initialData = [
  { id: '1', name: 'Alice Johnson', email: 'alice@example.com', role: 'Admin' },
  { id: '2', name: 'Bob Smith', email: 'bob@example.com', role: 'User' },
  { id: '3', name: 'Charlie Rose', email: 'charlie@example.com', role: 'Moderator' },
  { id: '4', name: 'Diana Prince', email: 'diana@example.com', role: 'Admin' },
  { id: '5', name: 'Ethan Hunt', email: 'ethan@example.com', role: 'User' },
  { id: '6', name: 'Fiona Glenanne', email: 'fiona@example.com', role: 'Moderator' },
  { id: '7', name: 'George Clooney', email: 'george@example.com', role: 'User' },
  { id: '8', name: 'Hannah Baker', email: 'hannah@example.com', role: 'Moderator' },
  { id: '9', name: 'Ian Fleming', email: 'ian@example.com', role: 'Admin' },
  { id: '10', name: 'Julia Child', email: 'julia@example.com', role: 'User' },
  { id: '11', name: 'Kevin Durant', email: 'kevin@example.com', role: 'Moderator' },
  { id: '12', name: 'Lara Croft', email: 'lara@example.com', role: 'Admin' },
];

export default function UsersList() {
  const [form] = Form.useForm();
  const [data, setData] = useState(initialData);
  const [editingKey, setEditingKey] = useState<string | null>(null);
  const navigate = useNavigate();

  const isEditing = (record: any) => record.id === editingKey;

  const edit = (record: any) => {
    form.setFieldsValue({ name: '', email: '', role: '', ...record });
    setEditingKey(record.id);
  };

  const cancel = () => setEditingKey(null);

  const save = async (id: string) => {
    const row = await form.validateFields();
    const newData = [...data];
    const idx = newData.findIndex((item) => item.id === id);
    if (idx > -1) {
      newData[idx] = { ...newData[idx], ...row };
      setData(newData);
      setEditingKey(null);
    }
  };

  const remove = (id: string) => {
    setData((prev) => prev.filter((item) => item.id !== id));
  };

  const columns = [
    {
      title: 'Name',
      dataIndex: 'name',
      key: 'name',
      onCell: (record: any) => ({
        record,
        inputType: 'text',
        dataIndex: 'name',
        title: 'Name',
        editing: isEditing(record),
      }),
    },
    {
      title: 'Email',
      dataIndex: 'email',
      key: 'email',
      onCell: (record: any) => ({
        record,
        inputType: 'text',
        dataIndex: 'email',
        title: 'Email',
        editing: isEditing(record),
      }),
    },
    {
      title: 'Role',
      dataIndex: 'role',
      key: 'role',
      onCell: (record: any) => ({
        record,
        inputType: 'select',
        dataIndex: 'role',
        title: 'Role',
        editing: isEditing(record),
      }),
      render: (role: string, record: any) =>
        isEditing(record) ? null : (
          <Tag color={role === 'Admin' ? 'volcano' : role === 'Moderator' ? 'geekblue' : 'green'}>
            {role}
          </Tag>
        ),
    },
    {
      title: 'Actions',
      key: 'actions',
      render: (_: any, record: any) => {
        const editable = isEditing(record);
        return editable ? (
          <Space>
            <Button
              icon={<SaveOutlined />}
              size="small"
              type="primary"
              onClick={(e) => {
                e.stopPropagation();
                save(record.id);
              }}
            />
            <Button
              icon={<CloseOutlined />}
              size="small"
              onClick={(e) => {
                e.stopPropagation();
                cancel();
              }}
            />
          </Space>
        ) : (
          <Space>
            <Button
              icon={<EditOutlined />}
              onClick={(e) => {
                e.stopPropagation(); // prevent row click
                edit(record);
              }}
              size="small"
            />

            <Button
              icon={<DeleteOutlined />}
              danger
              size="small"
              onClick={(e) => {
                e.stopPropagation();
                remove(record.id);
              }}
            />
          </Space>
        );
      },
    },
  ];

  const mergedColumns = columns.map((col) => ({
    ...col,
    onCell: col.onCell
      ? (record: any) => ({
          ...col.onCell!(record),
          editing: isEditing(record),
        })
      : undefined,
  }));

  const EditableCell: React.FC<any> = ({
    editing,
    dataIndex,
    title,
    inputType,
    record,
    children,
    ...restProps
  }) => {
    const inputNode =
      inputType === 'select' ? (
        <Select>
          <Option value="Admin">Admin</Option>
          <Option value="User">User</Option>
          <Option value="Moderator">Moderator</Option>
        </Select>
      ) : (
        <Input />
      );

    return (
      <td {...restProps}>
        {editing ? (
          <Form.Item
            name={dataIndex}
            style={{ margin: 0 }}
            rules={[{ required: true, message: `Please enter ${title}` }]}
          >
            {inputNode}
          </Form.Item>
        ) : (
          children
        )}
      </td>
    );
  };

  return (
    <DashboardLayout title="Users" description="A list of all the users.">
      <Form form={form} component={false}>
        <Table
          components={{ body: { cell: EditableCell as any } }}
          dataSource={data}
          columns={mergedColumns}
          rowKey="id"
          pagination={{ pageSize: 5 }}
          bordered
          onRow={(record) => ({
            onClick: () => {
              if (!isEditing(record)) {
                navigate(`/users/${record.id}`);
              }
            },
            style: { cursor: 'pointer' },
          })}
        />
      </Form>
    </DashboardLayout>
  );
}
