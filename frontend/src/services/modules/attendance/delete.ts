import { api } from "@/utils/api";

/** DELETE /api/modules/{module_id}/attendance/sessions/{session_id} */
export const deleteAttendanceSession = async (
  moduleId: number,
  sessionId: number
) => {
  return api.delete<null>(
    `/modules/${moduleId}/attendance/sessions/${sessionId}`
  );
};
