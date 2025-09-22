import type {
  PaginationResponse,
  Timestamp,
  PaginationRequest,
  SortOption,
} from "@/types/common";

/** DB-backed attendance session (+ student-only counts) */
export interface AttendanceSession extends Timestamp {
  id: number;
  module_id: number;
  created_by: number;
  title: string;
  rotation_seconds: number;
  restrict_by_ip: boolean;
  allowed_ip_cidr?: string | null;
  created_from_ip?: string | null;
  /** Session is currently enabled (controls whether codes can be shown/used) */
  active: boolean;

  /** how many students have attended this session */
  attended_count: number;
  /** total number of students enrolled in the module */
  student_count: number;
}

/** Query params for list endpoint */
export interface ListAttendanceSessionsParams extends Partial<PaginationRequest> {
  q?: string;
  /** Filter by enabled/disabled */
  active?: boolean;
  /**
   * Sort fields supported by the backend:
   *  - "created_at", "title", "active"
   * Prefix with '-' for descending (e.g. "-created_at").
   */
  sort?: SortOption[];
}

/** Create payload (matches backend CreateSessionReq) */
export interface CreateAttendanceSessionReq {
  title: string;
  active?: boolean;
  rotation_seconds?: number;
  restrict_by_ip?: boolean;
  allowed_ip_cidr?: string;
  pin_to_creator_ip?: boolean;
}

/** Edit payload (matches backend EditSessionReq) */
export interface EditAttendanceSessionReq {
  title?: string;
  active?: boolean;
  rotation_seconds?: number;
  restrict_by_ip?: boolean;
  allowed_ip_cidr?: string;
  created_from_ip?: string;
}

/** Data wrapper for list */
export interface ListAttendanceSessionsData extends PaginationResponse {
  sessions: AttendanceSession[];
}

/** Matches backend DTO for an attendance record */
export interface AttendanceRecord {
  session_id: number;
  user_id: number;
  username?: string | null;
  taken_at: string;          // ISO string
  ip_address?: string | null;
  token_window: number;
}

/** Query params for listing records of a session */
export interface ListAttendanceRecordsParams extends Partial<PaginationRequest> {
  /** free-text: numeric → user_id; text → username/ip contains */
  q?: string;
  /** sort by "taken_at" | "user_id" (prefix '-' for desc) */
  sort?: SortOption[]; // will serialize to a single sort string via buildQuery
}

/** Paged response for listing records of a session */
export interface ListAttendanceRecordsData extends PaginationResponse {
  records: AttendanceRecord[];
}
