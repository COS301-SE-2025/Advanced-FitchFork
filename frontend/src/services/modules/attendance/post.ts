import type {
  AttendanceSession,
  CreateAttendanceSessionReq,
} from "@/types/modules/attendance";
import { api } from "@/utils/api";

/** POST /api/modules/{module_id}/attendance/sessions */
export const createAttendanceSession = async (
  moduleId: number,
  payload: CreateAttendanceSessionReq
) => {
  return api.post<AttendanceSession>(
    `/modules/${moduleId}/attendance/sessions`,
    payload
  );
};

// services/modules/attendance/post.ts
export const markAttendance = (moduleId: number, sessionId: number, code: string, method: 'qr'|'manual' = 'manual') =>
  api.post<void>(`/modules/${moduleId}/attendance/sessions/${sessionId}/mark`, { code, method });

export const markAttendanceByUsername = (
  moduleId: number,
  sessionId: number,
  username: string,
) =>
  api.post<void>(
    `/modules/${moduleId}/attendance/sessions/${sessionId}/mark/by-username`,
    { username }
  );
