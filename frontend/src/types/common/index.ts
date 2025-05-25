export type SortOrder = 'ascend' | 'descend';

export interface SortOption {
    field: string,
    order: SortOrder;
}

export interface Timestamp {
    created_at: string,
    updated_at: string,
}