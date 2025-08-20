import { ASSIGNMENT_STATUSES, type AssignmentStatus } from "@/types/modules/assignments";

export function getRandomAssignmentStatus(): AssignmentStatus {
  const i = Math.floor(Math.random() * ASSIGNMENT_STATUSES.length);
  return ASSIGNMENT_STATUSES[i];
}