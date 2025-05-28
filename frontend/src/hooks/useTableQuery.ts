import { useState } from 'react';
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
    pageSize: 5,
    pageSizeOptions: [5, 10, 20, 50],
    total: 0,
    style: {
      paddingRight: '20px',
    },
  });

  const setPagination = (patch: Partial<PaginationState>) => {
    setPaginationState((prev) => ({ ...prev, ...patch }));
  };

  const clearSearch = () => setSearchTerm('');
  const clearSorters = () => setSorterState([]);
  const clearFilters = () => setFilterState({});

  const clearAll = () => {
    clearSearch();
    clearSorters();
    clearFilters();
    setPagination({ current: 1 });
  };
  
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
