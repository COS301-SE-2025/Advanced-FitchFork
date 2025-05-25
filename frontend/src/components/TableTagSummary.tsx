import { Tag } from 'antd';
import {
  SearchOutlined,
  FilterOutlined,
  SortAscendingOutlined,
  CloseOutlined,
} from '@ant-design/icons';

interface TableTagSummaryProps {
  searchTerm: string;
  onClearSearch: () => void;
  filters: Record<string, any>;
  onClearFilter: (key: string) => void;
  sorters: { columnKey: string; order: 'ascend' | 'descend' }[];
  onClearSorter: (key: string) => void;
}

const TableTagSummary: React.FC<TableTagSummaryProps> = ({
  searchTerm,
  onClearSearch,
  filters,
  onClearFilter,
  sorters,
  onClearSorter,
}) => {
  const hasFilters = Object.entries(filters).some(
    ([, val]) => Array.isArray(val) && val.length > 0,
  );
  const hasSorts = sorters.some((s) => s.order);

  if (!searchTerm && !hasFilters && !hasSorts) return null;

  return (
    <div className="mb-4 flex flex-wrap items-center gap-2">
      {/* Search Tag: Cyan */}
      {searchTerm && (
        <Tag
          color="cyan"
          closable
          onClose={onClearSearch}
          icon={<SearchOutlined style={{ color: 'currentColor' }} />}
          closeIcon={<CloseOutlined style={{ color: '#08979c' }} />}
        >
          {searchTerm}
        </Tag>
      )}

      {/* Filter Tags: Volcano */}
      {Object.entries(filters).map(
        ([key, value]) =>
          Array.isArray(value) &&
          value[0] && (
            <Tag
              key={key}
              color="volcano"
              closable
              onClose={() => onClearFilter(key)}
              icon={<FilterOutlined style={{ color: 'currentColor' }} />}
              closeIcon={<CloseOutlined style={{ color: '#d4380d' }} />}
            >
              {value[0]}
            </Tag>
          ),
      )}

      {/* Sort Tags: Gold */}
      {sorters
        .filter((s) => !!s.order)
        .map((s) => (
          <Tag
            key={s.columnKey}
            color="gold"
            closable
            onClose={() => onClearSorter(s.columnKey)}
            icon={<SortAscendingOutlined style={{ color: 'currentColor' }} />}
            closeIcon={<CloseOutlined style={{ color: '#d48806' }} />}
          >
            {s.columnKey} {s.order === 'ascend' ? '↑' : '↓'}
          </Tag>
        ))}
    </div>
  );
};

export default TableTagSummary;
