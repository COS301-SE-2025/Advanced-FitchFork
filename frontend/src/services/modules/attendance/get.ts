import type {
  AttendanceSession,
  ListAttendanceSessionsData,
  ListAttendanceSessionsParams,
  ListAttendanceRecordsParams,
  ListAttendanceRecordsData,
} from "@/types/modules/attendance";
import { api, apiDownload } from "@/utils/api";

/** GET /api/modules/{module_id}/attendance/sessions */
export const listAttendanceSessions = async (
  moduleId: number,
  params: ListAttendanceSessionsParams = {}
) => {
  return api.get<ListAttendanceSessionsData>(
    `/modules/${moduleId}/attendance/sessions`,
    params
  );
};

/** GET /api/modules/{module_id}/attendance/sessions/{session_id} */
export const getAttendanceSession = async (
  moduleId: number,
  sessionId: number
) => {
  return api.get<AttendanceSession>(
    `/modules/${moduleId}/attendance/sessions/${sessionId}`
  );
};

/** GET /api/modules/{module_id}/attendance/sessions/{session_id}/code */
export const getCurrentAttendanceCode = (moduleId: number, sessionId: number) =>
  api.get<string>(`/modules/${moduleId}/attendance/sessions/${sessionId}/code`);

/**
 * GET /modules/:moduleId/attendance/sessions/:sessionId/records
 * Paged list with search/sort.
 */
export const listAttendanceSessionRecords = async (
  moduleId: number,
  sessionId: number,
  params: ListAttendanceRecordsParams = {}
) => {
  return api.get<ListAttendanceRecordsData>(
    `/modules/${moduleId}/attendance/sessions/${sessionId}/records`,
    params
  );
};

/**
 * Optional: fetch the CSV as a Blob (if you need to preview or custom-handle it).
 */
export const downloadAttendanceSessionRecordsCsv = async (
  moduleId: number,
  sessionId: number
) => {
  return apiDownload(`/modules/${moduleId}/attendance/sessions/${sessionId}/records/export`);
};
