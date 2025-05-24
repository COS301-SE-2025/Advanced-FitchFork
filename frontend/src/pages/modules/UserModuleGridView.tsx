import { useAuth } from '@/context/AuthContext';
import AppLayout from '@/layouts/AppLayout';
import { mockModules } from '@/mocks/modules';
import type { ModuleRole } from '@/types/users';
import { Card, Row, Col, Typography, Tag, Space, Input, Select, Button } from 'antd';
import { useMemo, useState } from 'react';

const { Text } = Typography;
const { Search } = Input;

const roleColors: Record<ModuleRole, string> = {
  lecturer: 'geekblue',
  tutor: 'gold',
  student: 'green',
};

interface Props {
  filter: 'student' | 'tutor' | 'lecturer';
}

const UserModuleGridView: React.FC<Props> = ({ filter }) => {
  const { user } = useAuth();

  const allVisibleModules = useMemo(() => {
    return mockModules.filter((mod) =>
      user?.module_roles.some((r) => r.moduleId === mod.id && r.role === filter),
    );
  }, [user, filter]);

  const getRolesForModule = (moduleId: number) => {
    return user?.module_roles.filter((r) => r.moduleId === moduleId).map((r) => r.role) ?? [];
  };

  const [searchTerm, setSearchTerm] = useState('');
  const [sortKey, setSortKey] = useState<'code' | 'year'>('code');

  const filteredModules = useMemo(() => {
    const lower = searchTerm.toLowerCase();
    return [...allVisibleModules]
      .filter(
        (mod) =>
          mod.code.toLowerCase().includes(lower) || mod.description.toLowerCase().includes(lower),
      )
      .sort((a, b) => (sortKey === 'code' ? a.code.localeCompare(b.code) : a.year - b.year));
  }, [searchTerm, allVisibleModules, sortKey]);

  const handleClearFilters = () => {
    setSearchTerm('');
    setSortKey('code');
  };

  return (
    <AppLayout
      title="Your Modules"
      description="These are the modules assigned to you. Use the filters to find what you're looking for."
    >
      <div className="max-w-full overflow-x-hidden">
        {/* Filter + Sort Bar */}
        <div className="mb-6 flex flex-wrap items-end gap-4">
          <div className="min-w-[220px] max-w-[300px]">
            <label className="block mb-1 text-sm text-gray-600">Search</label>
            <Search
              placeholder="Search by code or description"
              allowClear
              onChange={(e) => setSearchTerm(e.target.value)}
              value={searchTerm}
            />
          </div>

          <div className="min-w-[180px]">
            <label className="block mb-1 text-sm text-gray-600">Sort</label>
            <Select
              value={sortKey}
              onChange={(value) => setSortKey(value)}
              style={{ width: '100%' }}
            >
              <Select.Option value="code">Sort by Code</Select.Option>
              <Select.Option value="year">Sort by Year</Select.Option>
            </Select>
          </div>

          <div>
            <label className="invisible block mb-1">Clear</label>
            <Button onClick={handleClearFilters}>Clear</Button>
          </div>
        </div>

        <div className="px-3 py-5">
          {/* Module Grid */}
          <Row gutter={[24, 24]} className="!justify-start">
            {filteredModules.map((mod) => {
              const roles = getRolesForModule(mod.id);
              return (
                <Col key={mod.id} xs={24} sm={12} md={8} lg={6}>
                  <Card
                    hoverable
                    className="rounded-2xl border border-gray-200 hover:border-blue-500 hover:bg-gray-50 transition-colors duration-200"
                  >
                    <div className="flex justify-between items-center mb-3">
                      <Text strong className="text-lg">
                        {mod.code}
                      </Text>
                      <Space size={[4, 0]} wrap>
                        {roles.map((role) => (
                          <Tag key={role} color={roleColors[role]}>
                            {role.charAt(0).toUpperCase() + role.slice(1)}
                          </Tag>
                        ))}
                      </Space>
                    </div>
                    <Text className="text-gray-700 block mb-2">{mod.description}</Text>
                    <Text type="secondary" className="text-sm">
                      Year: {mod.year}
                    </Text>
                  </Card>
                </Col>
              );
            })}
          </Row>
        </div>

        {filteredModules.length === 0 && (
          <div className="text-center text-gray-500 mt-10">
            No modules found matching your criteria.
          </div>
        )}
      </div>
    </AppLayout>
  );
};

export default UserModuleGridView;
