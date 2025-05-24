export interface SortOption {
    field: string,
    order: 'asc' | 'desc'
}

export interface Timestamp {
    created_at: string,
    updated_at: string,
}