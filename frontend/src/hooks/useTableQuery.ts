import { useCallback, useState } from 'react';
import type { SortOption } from '@/types/common';
import type { TablePaginationConfig } from 'antd';

export interface PaginationState  extends TablePaginationConfig {
  current: number;
  pageSize: number;
  total: number;
}

export interface UseTableQueryResult {
  searchTerm: string;
  setSearchTerm: (value: string) => void;

  filterState: Record<string, string[]>;
  setFilterState: (filters: Record<string, string[]>) => void;

  sorterState: SortOption[];
  setSorterState: (sorters: SortOption[]) => void;

  pagination: PaginationState;
  setPagination: (pagination: Partial<PaginationState>) => void;

  clearAll: () => void;
  clearSearch: () => void;
  clearSorters: () => void;
  clearFilters: () => void;
}

/**
 * Manages Ant Design table state: search, sort, filter, and pagination.
 */
export function useTableQuery(): UseTableQueryResult {
  const [searchTerm, setSearchTerm] = useState('');
  const [filterState, setFilterState] = useState<Record<string, string[]>>({});
  const [sorterState, setSorterState] = useState<SortOption[]>([]);
  const [pagination, setPaginationState] = useState<PaginationState>({
    current: 1,
    pageSize: 10,
    pageSizeOptions: [5, 10, 20, 50],
    total: 0,
    style: {
      paddingRight: '20px',
    },
  });

  const setPagination = useCallback((patch: Partial<PaginationState>) => {
    setPaginationState((prev) => ({ ...prev, ...patch }));
  }, []);

  const clearSearch = useCallback(() => setSearchTerm(''), []);
  const clearSorters = useCallback(() => setSorterState([]), []);
  const clearFilters = useCallback(() => setFilterState({}), []);

  const clearAll = useCallback(() => {
    setSearchTerm('');
    setSorterState([]);
    setFilterState({});
    setPaginationState((prev) => ({ ...prev, current: 1 }));
  }, []);
  
  return {
    searchTerm,
    setSearchTerm,
    filterState,
    setFilterState,
    sorterState,
    setSorterState,
    pagination,
    setPagination,
    clearAll,
    clearSearch,
    clearSorters,
    clearFilters,
  };
}
