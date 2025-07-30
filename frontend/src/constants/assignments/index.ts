import type { AssignmentConfig } from "@/types/modules/assignments/config";

export const DEFAULT_ASSIGNMENT_CONFIG: AssignmentConfig = {
  execution: {
    timeout_secs: 10,
    max_memory: 8589934592, // bytes
    max_cpus: 2,
    max_uncompressed_size: 100000000, // bytes
    max_processes: 256,
  },
  marking: {
    marking_scheme: 'exact',
    feedback_scheme: 'auto',
    deliminator: '&-=-&',
  },
};