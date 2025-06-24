/**
 * Standardized shape for all API responses.
 *
 * @template T - The expected type of the `data` payload.
 */
export interface ApiResponse<T> {
  /** Indicates if the request was successful. */
  success: boolean;

  /** The actual response data from the server. */
  data: T;

  /** A human-readable message from the server. */
  message: string;
}

export type SortOrder = 'ascend' | 'descend';

export interface SortOption {
    field: string,
    order: SortOrder;
}

export interface Timestamp {
    created_at: string,
    updated_at: string,
}

export interface Score {
    earned: number;
    total: number;
}

export interface PaginationRequest {
  page: number;
  per_page: number;
  sort?: SortOption[];
  query?: string;
}

export interface PaginationResponse {
  page: number;
  per_page: number;
  total: number;
}