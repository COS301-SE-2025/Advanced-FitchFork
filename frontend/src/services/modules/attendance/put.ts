import type {
  AttendanceSession,
  EditAttendanceSessionReq,
} from "@/types/modules/attendance";
import { api } from "@/utils/api";

/** PUT /api/modules/{module_id}/attendance/sessions/{session_id} */
export const editAttendanceSession = async (
  moduleId: number,
  sessionId: number,
  payload: EditAttendanceSessionReq
) => {
  return api.put<AttendanceSession>(
    `/modules/${moduleId}/attendance/sessions/${sessionId}`,
    payload
  );
};
