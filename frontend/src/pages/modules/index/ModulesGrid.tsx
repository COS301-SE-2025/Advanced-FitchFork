import { useState, useMemo } from 'react';
import {
  Button,
  Col,
  Divider,
  Dropdown,
  Empty,
  Input,
  Row,
  Typography,
  type MenuProps,
} from 'antd';
import {
  DownOutlined,
  FilterOutlined,
  SortAscendingOutlined,
  CheckOutlined,
} from '@ant-design/icons';
import ModuleCard from '@/components/modules/ModuleCard';
import type { Module } from '@/types/modules';
import PageHeader from '@/components/PageHeader';

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

interface Props {
  title: string;
  modules: Module[];
}

const ModulesGrid = ({ title, modules }: Props) => {
  const [activeSort, setActiveSort] = useState<string | null>(() => {
    return localStorage.getItem('modules_sort') || null;
  });
  const [activeFilter, setActiveFilter] = useState<string | null>(() => {
    return localStorage.getItem('modules_filter') || null;
  });
  const [searchText, setSearchText] = useState<string>(() => {
    return localStorage.getItem('modules_search') || '';
  });
  const [favorites, setFavorites] = useState<number[]>(() => {
    const stored = localStorage.getItem('module_favorites');
    return stored ? JSON.parse(stored) : [];
  });

  const handleSortSelect = (key: string) => {
    const newSort = activeSort === key ? null : key;
    setActiveSort(newSort);
    if (newSort) {
      localStorage.setItem('modules_sort', newSort);
    } else {
      localStorage.removeItem('modules_sort');
    }
  };

  const handleFilterSelect = (key: string) => {
    const newFilter = activeFilter === key ? null : key;
    setActiveFilter(newFilter);
    if (newFilter) {
      localStorage.setItem('modules_filter', newFilter);
    } else {
      localStorage.removeItem('modules_filter');
    }
  };

  const handleSearch = (value: string) => {
    const val = value.toLowerCase();
    setSearchText(val);
    if (val) {
      localStorage.setItem('modules_search', val);
    } else {
      localStorage.removeItem('modules_search');
    }
  };

  const handleClearFilters = () => {
    setSearchText('');
    setActiveSort(null);
    setActiveFilter(null);
    localStorage.removeItem('modules_search');
    localStorage.removeItem('modules_sort');
    localStorage.removeItem('modules_filter');
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

    if (activeFilter) {
      result = result.filter((m) => String(m.year) === activeFilter);
    }

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

    // ⬇️ Prioritize favorites (after other sorting)
    result.sort((a, b) => {
      const aFav = favorites.includes(a.id) ? 1 : 0;
      const bFav = favorites.includes(b.id) ? 1 : 0;
      return bFav - aFav;
    });

    return result;
  }, [modules, activeFilter, activeSort, searchText, favorites]);

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

  const favoriteModules = filteredSortedModules.filter((m) => favorites.includes(m.id));
  const otherModules = filteredSortedModules.filter((m) => !favorites.includes(m.id));

  return (
    <div className="p-4 sm:p-6">
      <PageHeader title={title} />

      <div>
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
                Filter <DownOutlined />
              </Button>
            </Dropdown>
          </div>
        </div>

        {filteredSortedModules.length === 0 ? (
          <div className="flex flex-col items-center gap-4">
            <Empty description="No modules found." />
            {(activeSort || activeFilter || searchText) && (
              <Button onClick={handleClearFilters}>Clear Filters</Button>
            )}
          </div>
        ) : (
          <>
            {favoriteModules.length > 0 && (
              <>
                <div className="py-4">
                  <Title level={4}>Favorites</Title>
                </div>
                <Row gutter={[24, 24]}>
                  {favoriteModules.map((module) => (
                    <Col key={module.id} xs={24} sm={24} md={12} lg={12} xl={8} xxl={6}>
                      <ModuleCard
                        module={module}
                        isFavorite={true}
                        onToggleFavorite={handleToggleFavorite}
                      />
                    </Col>
                  ))}
                </Row>
              </>
            )}

            {otherModules.length > 0 && (
              <>
                <div className="py-4">
                  <Title level={4}>Other Modules</Title>
                </div>

                <Row gutter={[24, 24]}>
                  {otherModules.map((module) => (
                    <Col key={module.id} xs={24} sm={24} md={12} lg={12} xl={8} xxl={6}>
                      <ModuleCard
                        module={module}
                        isFavorite={false}
                        onToggleFavorite={handleToggleFavorite}
                      />
                    </Col>
                  ))}
                </Row>
              </>
            )}
          </>
        )}
      </div>
    </div>
  );
};

export default ModulesGrid;
