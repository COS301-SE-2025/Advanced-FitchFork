import { apiFetchBlob, apiDownload } from "@/utils/api";

/** Fetch the first overwrite file as a Blob (no auto-download). */
export const fetchOverwriteFileBlob = async (
  moduleId: number,
  assignmentId: number,
  taskId: number
): Promise<Blob> => {
  return apiFetchBlob(
    `/modules/${moduleId}/assignments/${assignmentId}/overwrite_files/task/${taskId}`
  );
};

/** Download the first overwrite file (uses Content-Disposition filename). */
export const downloadOverwriteFile = async (
  moduleId: number,
  assignmentId: number,
  taskId: number
): Promise<void> => {
  return apiDownload(
    `/modules/${moduleId}/assignments/${assignmentId}/overwrite_files/task/${taskId}`
  );
};
