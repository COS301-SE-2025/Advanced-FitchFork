import { useState, useMemo } from 'react';
import { Button, Col, Dropdown, Empty, Input, Row, Typography, type MenuProps } from 'antd';
import {
  DownOutlined,
  FilterOutlined,
  SortAscendingOutlined,
  CheckOutlined,
} from '@ant-design/icons';
import ModuleCard from '@/components/modules/ModuleCard';
import { MODULE_ROLES, type ModuleRole } from '@/types/modules';
import PageHeader from '@/components/PageHeader';
import type { Module } from '@/types/modules';

const { Search } = Input;
const { Title } = Typography;

const sortMenuOptions = [
  { key: 'newest', label: 'Newest' },
  { key: 'oldest', label: 'Oldest' },
  { key: 'az', label: 'A-Z' },
  { key: 'za', label: 'Z-A' },
];

const filterOptions = [
  { key: '2023', label: '2023' },
  { key: '2024', label: '2024' },
  { key: '2025', label: '2025' },
];

interface UserModuleRole extends Module {
  role: ModuleRole;
}

interface Props {
  title: string;
  modules: UserModuleRole[];
}

const ModulesGrid = ({ title, modules }: Props) => {
  const [activeSort, setActiveSort] = useState<string | null>(
    () => localStorage.getItem('modules_sort') || null,
  );
  const [activeFilter, setActiveFilter] = useState<string | null>(
    () => localStorage.getItem('modules_filter') || null,
  );
  const [searchText, setSearchText] = useState<string>(
    () => localStorage.getItem('modules_search') || '',
  );
  const [favorites, setFavorites] = useState<number[]>(() => {
    const stored = localStorage.getItem('module_favorites');
    return stored ? JSON.parse(stored) : [];
  });
  const [activeRoles, setActiveRoles] = useState<ModuleRole[]>(() => {
    const stored = localStorage.getItem('modules_roles');
    return stored ? JSON.parse(stored) : [];
  });

  const [visibleRoles, setVisibleRoles] = useState<ModuleRole[]>(() => {
    const stored = localStorage.getItem('modules_visible_roles');
    return stored ? JSON.parse(stored) : ['Lecturer', 'Tutor', 'Student'];
  });

  const handleSortSelect = (key: string) => {
    const newSort = activeSort === key ? null : key;
    setActiveSort(newSort);
    newSort
      ? localStorage.setItem('modules_sort', newSort)
      : localStorage.removeItem('modules_sort');
  };

  const handleFilterSelect = (key: string) => {
    const newFilter = activeFilter === key ? null : key;
    setActiveFilter(newFilter);
    newFilter
      ? localStorage.setItem('modules_filter', newFilter)
      : localStorage.removeItem('modules_filter');
  };

  const handleRoleFilterToggle = (role: ModuleRole) => {
    const updated = activeRoles.includes(role)
      ? activeRoles.filter((r) => r !== role)
      : [...activeRoles, role];
    setActiveRoles(updated);
    localStorage.setItem('modules_roles', JSON.stringify(updated));
  };

  const handleSearch = (value: string) => {
    const val = value.toLowerCase();
    setSearchText(val);
    val ? localStorage.setItem('modules_search', val) : localStorage.removeItem('modules_search');
  };

  const handleClearFilters = () => {
    setSearchText('');
    setActiveSort(null);
    setActiveFilter(null);
    setActiveRoles([]);
    localStorage.removeItem('modules_search');
    localStorage.removeItem('modules_sort');
    localStorage.removeItem('modules_filter');
    localStorage.removeItem('modules_roles');
  };

  const handleToggleFavorite = (moduleId: number) => {
    const updated = favorites.includes(moduleId)
      ? favorites.filter((id) => id !== moduleId)
      : [...favorites, moduleId];
    setFavorites(updated);
    localStorage.setItem('module_favorites', JSON.stringify(updated));
  };

  const filteredSortedModules = useMemo(() => {
    let result = [...modules];

    if (activeFilter) result = result.filter((m) => String(m.year) === activeFilter);
    if (activeRoles.length > 0) result = result.filter((m) => activeRoles.includes(m.role));
    if (searchText.trim()) {
      result = result.filter(
        (m) =>
          m.code.toLowerCase().includes(searchText) ||
          m.description.toLowerCase().includes(searchText),
      );
    }

    switch (activeSort) {
      case 'newest':
        result.sort((a, b) => b.year - a.year);
        break;
      case 'oldest':
        result.sort((a, b) => a.year - b.year);
        break;
      case 'az':
        result.sort((a, b) => a.code.localeCompare(b.code));
        break;
      case 'za':
        result.sort((a, b) => b.code.localeCompare(a.code));
        break;
    }

    result.sort((a, b) => {
      const aFav = favorites.includes(a.id) ? 1 : 0;
      const bFav = favorites.includes(b.id) ? 1 : 0;
      return bFav - aFav;
    });

    return result;
  }, [modules, activeFilter, activeSort, activeRoles, searchText, favorites]);

  const favoriteModules = useMemo(() => {
    return filteredSortedModules.filter((m) => favorites.includes(m.id));
  }, [filteredSortedModules, favorites]);

  const groupedByRole = useMemo(() => {
    const groups: Partial<Record<ModuleRole, UserModuleRole[]>> = {
      Lecturer: [],
      Tutor: [],
      Student: [],
    };
    for (const m of filteredSortedModules) {
      if (!favorites.includes(m.id)) {
        groups[m.role]?.push(m);
      }
    }

    return groups;
  }, [filteredSortedModules]);

  const sortMenuItems: MenuProps['items'] = sortMenuOptions.map((opt) => ({
    key: opt.key,
    label: (
      <div className="flex items-center justify-between" onClick={() => handleSortSelect(opt.key)}>
        {opt.label} {activeSort === opt.key && <CheckOutlined />}
      </div>
    ),
  }));

  const filterMenuItems: MenuProps['items'] = filterOptions.map((opt) => ({
    key: opt.key,
    label: (
      <div
        className="flex items-center justify-between"
        onClick={() => handleFilterSelect(opt.key)}
      >
        {opt.label} {activeFilter === opt.key && <CheckOutlined />}
      </div>
    ),
  }));

  const roleFilterMenuItems: MenuProps['items'] = MODULE_ROLES.map((role) => ({
    key: role,
    label: (
      <div
        className="flex items-center justify-between"
        onClick={() => handleRoleFilterToggle(role)}
      >
        {role} {activeRoles.includes(role) && <CheckOutlined />}
      </div>
    ),
  }));

  const visibleSectionLabels: Record<ModuleRole, string> = {
    Lecturer: 'Lecturing',
    Tutor: 'Tutoring',
    Student: 'Enrolled',
  };

  return (
    <div className="bg-white dark:bg-gray-950 p-4 sm:p-6 h-full overflow-y-auto">
      <PageHeader title={title} description="View all your enrolled modules here." />

      <div className="mb-4 flex flex-wrap items-center gap-4">
        <Search
          placeholder="Search modules"
          allowClear
          onSearch={handleSearch}
          onChange={(e) => handleSearch(e.target.value)}
          className="max-w-xs sm:max-w-sm md:max-w-md w-full"
        />

        <div className="flex flex-wrap gap-2">
          <Dropdown menu={{ items: sortMenuItems }} trigger={['click']}>
            <Button icon={<SortAscendingOutlined />}>
              Sort <DownOutlined />
            </Button>
          </Dropdown>
          <Dropdown menu={{ items: filterMenuItems }} trigger={['click']}>
            <Button icon={<FilterOutlined />}>
              Year <DownOutlined />
            </Button>
          </Dropdown>
          <Dropdown menu={{ items: roleFilterMenuItems }} trigger={['click']}>
            <Button icon={<FilterOutlined />}>
              Role <DownOutlined />
            </Button>
          </Dropdown>
        </div>
        <Dropdown
          trigger={['click']}
          menu={{
            items: (['Lecturer', 'Tutor', 'Student'] as ModuleRole[])
              .filter((role) => groupedByRole[role]?.length)
              .map((role) => {
                const isVisible = visibleRoles.includes(role);
                return {
                  key: role,
                  label: (
                    <div
                      className="flex items-center justify-between w-36"
                      onClick={() => {
                        const updated = isVisible
                          ? visibleRoles.filter((r) => r !== role)
                          : [...visibleRoles, role];
                        setVisibleRoles(updated);
                        localStorage.setItem('modules_visible_roles', JSON.stringify(updated));
                      }}
                    >
                      {visibleSectionLabels[role]} {isVisible && <CheckOutlined />}
                    </div>
                  ),
                };
              }),
          }}
        >
          <Button icon={<FilterOutlined />}>
            Visible Sections <DownOutlined />
          </Button>
        </Dropdown>
      </div>

      {favoriteModules.length > 0 && (
        <div className="mb-6">
          <Title level={4}>Favorites</Title>
          <Row gutter={[24, 24]}>
            {favoriteModules.map((module) => (
              <Col key={module.id} xs={24} sm={12} md={12} lg={8} xl={6} xxl={6}>
                <ModuleCard module={module} isFavorite onToggleFavorite={handleToggleFavorite} />
              </Col>
            ))}
          </Row>
        </div>
      )}

      {filteredSortedModules.length === 0 ? (
        <div className="flex flex-col items-center gap-4">
          <Empty description="No modules found." />
          {(activeSort || activeFilter || searchText || activeRoles.length > 0) && (
            <Button onClick={handleClearFilters}>Clear Filters</Button>
          )}
        </div>
      ) : (
        <>
          {(['Lecturer', 'Tutor', 'Student'] as const).map((role) => {
            const list = groupedByRole[role];
            if (!list || list.length === 0) return null;

            const labelMap: Record<ModuleRole, string> = {
              Lecturer: 'Lecturing Modules',
              Tutor: 'Tutoring Modules',
              Student: 'Enrolled Modules',
            };

            if (!visibleRoles.includes(role)) {
              return null;
            }

            return (
              <div key={role} className="mb-10 pt-6 border-gray-200 dark:border-neutral-700">
                <div className="flex justify-between items-center mb-4">
                  <Title level={4} className="m-0 text-gray-800 dark:text-gray-200">
                    {labelMap[role]}
                  </Title>
                </div>
                <Row gutter={[16, 16]}>
                  {list.map((module) => (
                    <Col key={module.id} xs={24} sm={12} md={12} lg={8} xl={6} xxl={6}>
                      <ModuleCard
                        module={module}
                        isFavorite={favorites.includes(module.id)}
                        onToggleFavorite={handleToggleFavorite}
                      />
                    </Col>
                  ))}
                </Row>
              </div>
            );
          })}
        </>
      )}
    </div>
  );
};

export default ModulesGrid;
