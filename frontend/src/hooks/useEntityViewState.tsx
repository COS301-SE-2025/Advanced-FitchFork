import { useEffect, useState } from 'react';
import type { SortOption } from '@/types/common';
import type { TablePaginationConfig } from 'antd';
import { PAGE_SIZE_OPTIONS } from '@/constants/pagination';

/**
 * Pagination state that extends Ant Design's pagination config,
 * including required fields: current page, page size, and total items.
 */
export interface PaginationState extends TablePaginationConfig {
  current: number;
  pageSize: number;
  total: number;
}

/**
 * Options for configuring the entity view state hook.
 * @param viewModeKey Key to store/retrieve the preferred view mode in localStorage.
 * @param getInitialNewItem Function that returns an empty or default new item for creation modals.
 */
export interface UseEntityViewStateOptions<T> {
  viewModeKey: string;
  getInitialNewItem: () => Partial<T>;
  defaultViewMode?: 'table' | 'grid';
}

/**
 * Manages state for list views with table/grid modes, including:
 * - pagination
 * - search
 * - sorting
 * - filtering
 * - modal visibility
 * - selected items
 *
 * @param options View mode key and new item factory
 * @returns A state object and setters for managing list view UIs
 */
export function useEntityViewState<T>(options: UseEntityViewStateOptions<T>) {
  // ===========================
  // Search, Filter, Sort, Pagination
  // ===========================

  /** Search input state */
  const [searchTerm, setSearchTerm] = useState('');

  /** Column filter values */
  const [filterState, setFilterState] = useState<Record<string, string[]>>({});

  /** Column sorter state */
  const [sorterState, setSorterState] = useState<SortOption[]>([]);

  /** Pagination state */
  const [pagination, setPaginationState] = useState<PaginationState>({
    current: 1,
    pageSize: 10,
    total: 0,
    pageSizeOptions: PAGE_SIZE_OPTIONS,
    style: { paddingRight: '20px' },
  });

  /** Updates only part of the pagination state */
  const setPagination = (patch: Partial<PaginationState>) => {
    setPaginationState((prev) => ({ ...prev, ...patch }));
  };

  /** Clears the search term */
  const clearSearch = () => setSearchTerm('');

  /** Clears all active sorters */
  const clearSorters = () => setSorterState([]);

  /** Clears all applied filters */
  const clearFilters = () => setFilterState({});

  /** Clears search, filters, sorters, and resets page to 1 */
  const clearAll = () => {
    clearSearch();
    clearSorters();
    clearFilters();
    setPagination({ current: 1 });
  };

  // ===========================
  // View Mode (table/grid)
  // ===========================

  /** Current view mode: 'table' or 'grid' */
  const [viewMode, setViewModeInternal] = useState<'table' | 'grid'>(
    options.defaultViewMode || 'table',
  );

  /** Load and auto-switch view mode based on screen size */
  useEffect(() => {
    const handleResize = () => {
      const width = window.innerWidth;
      if (width < 640) {
        setViewModeInternal('grid');
      } else {
        const stored = localStorage.getItem(options.viewModeKey);
        if (stored === 'table' || stored === 'grid') {
          setViewModeInternal(stored);
        } else if (options.defaultViewMode) {
          setViewModeInternal(options.defaultViewMode);
          localStorage.setItem(options.viewModeKey, options.defaultViewMode);
        } else {
          setViewModeInternal('table');
        }
      }
    };

    window.addEventListener('resize', handleResize);
    handleResize();

    return () => window.removeEventListener('resize', handleResize);
  }, [options.viewModeKey, options.defaultViewMode]);

  /** Persist and set view mode */
  const setViewMode = (mode: 'table' | 'grid') => {
    setViewModeInternal(mode);
    localStorage.setItem(options.viewModeKey, mode);
  };

  // ===========================
  // Modals and Item States
  // ===========================

  /** Selected table row keys */
  const [selectedRowKeys, setSelectedRowKeys] = useState<React.Key[]>([]);

  /** Edit modal open state */
  const [editModalOpen, setEditModalOpen] = useState(false);

  /** Item being edited */
  const [editingItem, setEditingItem] = useState<T | null>(null);

  /** Create modal open state */
  const [isAddModalOpen, setIsAddModalOpen] = useState(false);

  /** New item state for creation */
  const [newItem, setNewItem] = useState<Partial<T>>(options.getInitialNewItem());

  return {
    // View mode
    viewMode,
    setViewMode,

    // Table state
    searchTerm,
    setSearchTerm,
    filterState,
    setFilterState,
    sorterState,
    setSorterState,
    pagination,
    setPagination,
    clearSearch,
    clearSorters,
    clearFilters,
    clearAll,

    // Row selection
    selectedRowKeys,
    setSelectedRowKeys,

    // Modal states
    editModalOpen,
    setEditModalOpen,
    editingItem,
    setEditingItem,
    isAddModalOpen,
    setIsAddModalOpen,
    newItem,
    setNewItem,
  };
}
